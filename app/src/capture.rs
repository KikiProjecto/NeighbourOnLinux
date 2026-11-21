use crate::{
    LaunchOptions,
    player::{MoveInput, Player},
    world::{Aabb, FLOOR_Y, HousePlan, STOREY},
};
use bevy::{
    prelude::*,
    render::{
        Render, RenderApp, RenderSystems,
        render_resource::{CachedPipelineState, PipelineCache},
        view::screenshot::{Screenshot, ScreenshotCaptured},
    },
};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

#[derive(Resource)]
pub struct CaptureTarget(pub Handle<Image>);
#[derive(Resource, Clone, Default)]
pub(crate) struct RenderReadiness {
    ready: Arc<AtomicBool>,
    failed: Arc<AtomicBool>,
}
impl RenderReadiness {
    pub(crate) fn is_ready(&self) -> bool {
        self.ready.load(Ordering::Acquire)
    }
}
#[derive(Resource, Default)]
struct CaptureStatus {
    frame: u32,
    requested: bool,
    started: Option<Instant>,
}
pub struct CapturePlugin;
impl Plugin for CapturePlugin {
    fn build(&self, app: &mut App) {
        let readiness = RenderReadiness::default();
        app.insert_resource(readiness.clone())
            .init_resource::<CaptureStatus>()
            .add_systems(Update, capture_or_exit);
        app.sub_app_mut(RenderApp)
            .insert_resource(readiness)
            .add_systems(Render, publish_readiness.after(RenderSystems::Render));
    }
}
fn capture_or_exit(
    mut commands: Commands,
    options: Res<LaunchOptions>,
    target: Option<Res<CaptureTarget>>,
    readiness: Res<RenderReadiness>,
    mut status: ResMut<CaptureStatus>,
    mut exit: MessageWriter<AppExit>,
) {
    status.frame += 1;
    let started = *status.started.get_or_insert_with(Instant::now);
    if let Some(path) = &options.screenshot {
        if readiness.failed.load(Ordering::Acquire) {
            error!("A GPU pipeline failed while preparing screenshot");
            exit.write(AppExit::error());
            return;
        }
        if status.frame >= 50 && readiness.ready.load(Ordering::Acquire) && !status.requested {
            let Some(target) = target else {
                return;
            };
            let path = path.clone();
            status.requested = true;
            commands.spawn(Screenshot::image(target.0.clone())).observe(
                move |capture: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                    let result = capture
                        .image
                        .clone()
                        .try_into_dynamic()
                        .map_err(|e| e.to_string())
                        .and_then(|image| image.save(&path).map_err(|e| e.to_string()));
                    match result {
                        Ok(()) => {
                            println!("Screenshot saved: {path}");
                            exit.write(AppExit::Success);
                        }
                        Err(error) => {
                            error!("Screenshot failed: {error}");
                            exit.write(AppExit::error());
                        }
                    }
                },
            );
        }
        if started.elapsed() > Duration::from_secs(120) {
            error!("Screenshot capture timed out");
            exit.write(AppExit::error());
        }
    } else if options.frames.is_some_and(|limit| status.frame >= limit) {
        exit.write(AppExit::Success);
    }
}

/// Publish only after actual pipelines have compiled, then settle across render frames.
/// The atomic is shared across Bevy's pipelined render thread, not a guessed sleep.
fn publish_readiness(
    cache: Res<PipelineCache>,
    readiness: Res<RenderReadiness>,
    mut settled: Local<(usize, u32)>,
) {
    let mut count = 0;
    let mut all_ready = true;
    let mut failed = false;
    for pipeline in cache.pipelines() {
        count += 1;
        match &pipeline.state {
            CachedPipelineState::Ok(_) => {}
            CachedPipelineState::Err(
                bevy::shader::ShaderCacheError::ShaderNotLoaded(_)
                | bevy::shader::ShaderCacheError::ShaderImportNotYetAvailable,
            ) => all_ready = false,
            CachedPipelineState::Err(error) => {
                failed = true;
                all_ready = false;
                error!("GPU shader compilation failed: {error}");
            }
            _ => all_ready = false,
        }
    }
    if all_ready && count > 0 && cache.waiting_pipelines().next().is_none() {
        settled.1 = if settled.0 == count {
            settled.1.saturating_add(1)
        } else {
            0
        };
    } else {
        settled.1 = 0;
    }
    settled.0 = count;
    readiness.ready.store(settled.1 >= 4, Ordering::Release);
    readiness.failed.store(failed, Ordering::Release);
}

/// Windowless smoke checks are also available to packaging scripts.
pub fn self_test() {
    let plan = HousePlan::default();
    let floor = Aabb::from_center(Vec3::new(0., -0.25, 0.), Vec3::new(20., 0.5, 20.));
    let wall = Aabb::from_center(Vec3::new(1., 1., 0.), Vec3::new(0.2, 2., 8.));
    let mut player = Player::new(Vec3::ZERO);
    let mut input = MoveInput {
        forward: 0.,
        strafe: 1.,
        jump: false,
        run: false,
        crouch: false,
        look: Vec2::ZERO,
    };
    for _ in 0..120 {
        player.step(1. / 60., &input, &[floor, wall]);
    }
    assert!(player.pos.x < 0.56, "player passed through wall");
    assert!(player.on_ground);
    input.strafe = 0.;
    input.jump = true;
    player.step(1. / 60., &input, &[floor]);
    assert!(player.vel.y > 0.);
    assert!(
        !plan
            .route(
                Vec3::new(0., 0., 12.),
                Vec3::new(4.8, FLOOR_Y + STOREY * 5., 5.2)
            )
            .is_empty()
    );
    let mut progress = crate::items::PuzzleProgress::default();
    let mut inventory = crate::items::Inventory {
        crowbar: true,
        ..default()
    };
    assert!(progress.disable_generator().is_err());
    progress.windmill_stopped = true;
    assert!(progress.disable_generator().is_ok());
    assert!(progress.take_keycard(&mut inventory).is_ok());
    assert_eq!(progress.open_basement(&inventory), Ok(false));
    assert_eq!(progress.open_basement(&inventory), Ok(true));
    for room in crate::basement::ROOMS {
        assert!(
            !plan
                .route(crate::world::PLAYER_SPAWN, room.center())
                .is_empty()
        );
    }
    let mut clock = crate::atmosphere::WorldClock::default();
    clock.advance(clock.day_length);
    assert_eq!(clock.day, 2);
    println!(
        "self-test ok · Bevy · {} pieces · {} colliders · {} rooms · six floors · day/night · 12-chamber underground labyrinth",
        plan.pieces.len(),
        plan.colliders.len(),
        plan.rooms.len()
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn standalone_smoke_checks() {
        self_test();
    }
    #[test]
    fn actual_house_stairs_are_climbable_without_head_collisions() {
        let plan = HousePlan::default();
        let mut p = Player::new(Vec3::new(6.5, FLOOR_Y, 5.2));
        for level in 0..5 {
            let x = if level % 2 == 0 { 6.5 } else { 9.6 };
            let start_z = if level % 2 == 0 { 5.2 } else { -7.2 };
            let end_z = if level % 2 == 0 { -7.2 } else { 5.2 };
            let current_y = p.pos.y;
            walk_to(&mut p, Vec3::new(x, current_y, start_z), &plan.colliders);
            walk_to(
                &mut p,
                Vec3::new(x, FLOOR_Y + (level + 1) as f32 * STOREY, end_z),
                &plan.colliders,
            );
            let expected = FLOOR_Y + (level + 1) as f32 * STOREY;
            assert!(
                (p.pos.y - expected).abs() < 0.15,
                "stuck on floor {level}: {:?}, expected {expected}",
                p.pos
            );
        }
    }
    fn walk_to(p: &mut Player, target: Vec3, colliders: &[Aabb]) {
        for _ in 0..500 {
            let delta = Vec3::new(target.x - p.pos.x, 0., target.z - p.pos.z);
            if delta.length() < 0.12 {
                p.vel.x = 0.;
                p.vel.z = 0.;
                return;
            }
            p.yaw = (-delta.x).atan2(-delta.z);
            p.step(
                1. / 60.,
                &MoveInput {
                    forward: 1.,
                    strafe: 0.,
                    jump: false,
                    run: false,
                    crouch: false,
                    look: Vec2::ZERO,
                },
                colliders,
            );
        }
        panic!("Could not walk to {target:?}, stuck at {:?}", p.pos);
    }
}
