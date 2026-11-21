//! Opt-in, local-only encounter replay and CPU/GPU timing capture. No work in normal play.
use crate::{
    LaunchOptions,
    ai::{AlertState, Neighbor},
    game::{GameSet, Outcome, Session},
    player::Player,
    world::{FLOOR_Y, PLAYER_SPAWN},
};
use bevy::{diagnostic::DiagnosticsStore, prelude::*, render::diagnostic::RenderDiagnosticsPlugin};
use std::{collections::BTreeMap, fmt::Write, time::Instant};

const WARMUP_SECONDS: f32 = 4.;
const CAPTURE_SECONDS: f32 = 28.;

pub struct EncounterProfilePlugin;
impl Plugin for EncounterProfilePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Capture>()
            .init_resource::<DiagnosticsStore>()
            .add_systems(First, begin_frame)
            .add_systems(
                PreUpdate,
                replay
                    .after(bevy::input::InputSystems)
                    .before(GameSet::Controls),
            )
            .add_systems(Last, finish_frame);
        // Timestamp/readback instrumentation can perturb integrated GPUs badly.
        // Keep ordinary replay timings representative; opt into pass-level data.
        if std::env::var("NEIGHBOR_PROFILE_GPU").is_ok_and(|value| value == "1") {
            app.add_plugins(RenderDiagnosticsPlugin);
        }
    }
}
#[derive(Resource)]
struct Capture {
    started: Instant,
    previous: Instant,
    frame_start: Instant,
    frame_ms: f64,
    elapsed: f32,
    cycle: Option<u32>,
    encounter: Option<u32>,
    done: bool,
    samples: BTreeMap<(&'static str, String), Vec<f64>>,
}
impl Default for Capture {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            started: now,
            previous: now,
            frame_start: now,
            frame_ms: 0.,
            elapsed: 0.,
            cycle: None,
            encounter: None,
            done: false,
            samples: default(),
        }
    }
}
fn begin_frame(mut capture: ResMut<Capture>, readiness: Res<crate::capture::RenderReadiness>) {
    let now = Instant::now();
    // Reset after Startup so shader/sample setup is outside the warm replay.
    if capture.cycle.is_none() || !readiness.is_ready() {
        capture.started = now;
    }
    capture.elapsed = now.duration_since(capture.started).as_secs_f32();
    capture.frame_ms = now.duration_since(capture.previous).as_secs_f64() * 1000.;
    capture.frame_start = now;
    capture.previous = now;
}
fn replay(
    mut capture: ResMut<Capture>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    mut players: Query<&mut Player>,
    mut neighbors: Query<&mut Neighbor>,
) {
    let (Ok(mut player), Ok(mut neighbor)) = (players.single_mut(), neighbors.single_mut()) else {
        return;
    };
    session.paused = false;
    let cycle = (capture.elapsed / 6.) as u32;
    let phase = capture.elapsed.rem_euclid(6.);
    keys.release(KeyCode::KeyR);
    if capture.cycle != Some(cycle) {
        keys.press(KeyCode::KeyR); // The actual retry handler resets the attempt.
        capture.cycle = Some(cycle);
    }
    if phase < 2. {
        player.pos = PLAYER_SPAWN;
        player.vel = Vec3::ZERO;
        player.yaw = 0.;
        player.pitch = 0.;
    } else if capture.encounter != Some(cycle) {
        *player = Player::new(Vec3::new(0., FLOOR_Y, 12.));
        neighbor.reset(None);
        neighbor.body.pos = Vec3::new(0., FLOOR_Y, 7.);
        neighbor.facing = Vec3::Z;
        neighbor.awareness = 0.9;
        capture.encounter = Some(cycle);
    }
}
fn finish_frame(
    mut capture: ResMut<Capture>,
    options: Res<LaunchOptions>,
    session: Res<Session>,
    neighbors: Query<&Neighbor>,
    diagnostics: Res<DiagnosticsStore>,
    mut exit: MessageWriter<AppExit>,
) {
    if capture.done || capture.elapsed < WARMUP_SECONDS {
        return;
    }
    let phase = if session.outcome == Outcome::Caught {
        "caught"
    } else if neighbors
        .single()
        .is_ok_and(|n| n.state == AlertState::Chase)
    {
        "chase"
    } else {
        "patrol"
    };
    let frame_ms = capture.frame_ms;
    let main_ms = capture.frame_start.elapsed().as_secs_f64() * 1000.;
    capture
        .samples
        .entry((phase, "frame_interval".into()))
        .or_default()
        .push(frame_ms);
    capture
        .samples
        .entry((phase, "main_schedule".into()))
        .or_default()
        .push(main_ms);
    for diagnostic in diagnostics.iter() {
        let path = diagnostic.path().as_str();
        if (path.ends_with("elapsed_gpu") || path.ends_with("elapsed_cpu"))
            && let Some(value) = diagnostic.value()
            && value.is_finite()
        {
            capture
                .samples
                .entry((phase, path.into()))
                .or_default()
                .push(value);
        }
    }
    if capture.elapsed < CAPTURE_SECONDS {
        return;
    }
    capture.done = true;
    let mut csv = String::from("phase,metric,samples,mean_ms,p50_ms,p95_ms,p99_ms,max_ms\n");
    for ((phase, metric), values) in &mut capture.samples {
        values.sort_by(f64::total_cmp);
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let _ = writeln!(
            csv,
            "{phase},{metric},{},{mean:.4},{:.4},{:.4},{:.4},{:.4}",
            values.len(),
            percentile(values, 0.5),
            percentile(values, 0.95),
            percentile(values, 0.99),
            values.last().unwrap()
        );
    }
    match std::fs::write(options.profile_encounter.as_ref().unwrap(), csv) {
        Ok(()) => {
            info!("Encounter profile saved after {:.1}s", capture.elapsed);
            exit.write(AppExit::Success);
        }
        Err(error) => {
            error!("Cannot save encounter profile: {error}");
            exit.write(AppExit::error());
        }
    }
}
fn percentile(sorted: &[f64], fraction: f64) -> f64 {
    sorted[((sorted.len() - 1) as f64 * fraction).round() as usize]
}
