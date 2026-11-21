//! Small bounded impact puffs and restrained leaf motion, driven by game state.
use crate::{
    audio::SoundCue,
    game::{Art, GameSet, Outcome, Session},
    world::{Paint, Piece, Shape},
};
use bevy::prelude::*;

pub struct EffectsPlugin;
impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_impacts, animate)
                .chain()
                .after(GameSet::Presentation),
        );
    }
}
#[derive(Component)]
pub(crate) struct LeafSway {
    rest: Transform,
    phase: f32,
}
#[derive(Component)]
struct Dust {
    velocity: Vec3,
    remaining: f32,
    attempt: u32,
    size: f32,
}
pub(crate) fn decorate(commands: &mut Commands, entity: Entity, piece: &Piece, seed: usize) {
    if piece.name == "Angular tree canopy" {
        commands.entity(entity).insert(LeafSway {
            rest: piece.transform,
            phase: seed as f32 * 1.7,
        });
    }
}
fn spawn_impacts(
    mut commands: Commands,
    mut cues: MessageReader<SoundCue>,
    art: Res<Art>,
    session: Res<Session>,
    dust: Query<(), With<Dust>>,
) {
    let mut capacity = 128_usize.saturating_sub(dust.iter().len());
    for cue in cues.read() {
        let SoundCue::Impact(position) = cue else {
            continue;
        };
        if session.paused || session.outcome != Outcome::Exploring {
            continue;
        }
        for i in 0..6.min(capacity) {
            let angle = i as f32 * std::f32::consts::TAU / 6.;
            let size = 0.025 + (i % 3) as f32 * 0.012;
            let entity = art.spawn(
                &mut commands,
                &Piece {
                    name: "Impact dust",
                    paint: Paint::Cream,
                    shape: Shape::Sphere,
                    transform: Transform::from_translation(*position + Vec3::Y * 0.03)
                        .with_scale(Vec3::splat(size)),
                },
            );
            commands.entity(entity).insert(Dust {
                velocity: Vec3::new(angle.sin() * 0.7, 0.3 + i as f32 * 0.04, angle.cos() * 0.7),
                remaining: 0.5,
                attempt: session.attempts,
                size,
            });
        }
        capacity = capacity.saturating_sub(6);
    }
}
fn animate(
    mut commands: Commands,
    time: Res<Time>,
    session: Res<Session>,
    mut age: Local<f32>,
    mut dust: Query<(Entity, &mut Dust, &mut Transform)>,
    mut leaves: Query<(&LeafSway, &mut Transform), Without<Dust>>,
) {
    if session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    let dt = time.delta_secs();
    *age += dt;
    for (entity, mut dust, mut tf) in &mut dust {
        dust.remaining -= dt;
        if dust.remaining <= 0. || dust.attempt != session.attempts {
            commands.entity(entity).despawn();
            continue;
        }
        tf.translation += dust.velocity * dt;
        dust.velocity *= (-3. * dt).exp();
        tf.scale = Vec3::splat(dust.size * (dust.remaining / 0.5).sqrt());
    }
    for (leaf, mut tf) in &mut leaves {
        *tf = leaf.rest;
        tf.rotate_z((*age * 0.8 + leaf.phase).sin() * 0.014);
        tf.translation.x += (*age * 0.6 + leaf.phase).sin() * 0.035;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn impact_bursts_are_capped_pause_and_expire() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f32(1. / 60.),
            ))
            .init_resource::<Session>()
            .insert_resource(Art::test_fixture())
            .add_message::<SoundCue>()
            .add_plugins(EffectsPlugin);
        app.update();
        for _ in 0..100 {
            app.world_mut().write_message(SoundCue::Impact(Vec3::ZERO));
        }
        app.update();
        let count = app.world_mut().query::<&Dust>().iter(app.world()).count();
        assert_eq!(count, 128);
        app.world_mut().resource_mut::<Session>().paused = true;
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(
            app.world_mut().query::<&Dust>().iter(app.world()).count(),
            count
        );
        app.world_mut().resource_mut::<Session>().paused = false;
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(
            app.world_mut().query::<&Dust>().iter(app.world()).count(),
            0
        );
    }
}
