//! Original synthesized audio with distance/occlusion mixing and managed pause.
use crate::{
    LaunchOptions,
    atmosphere::WorldClock,
    game::{GameSet, Outcome, Session, Train},
    items::PuzzleProgress,
    player::Player,
    props::{InteractiveProp, PropKind},
    world::{CollisionWorld, GENERATOR, WINDMILL_SWITCH, line_of_sight},
};
use bevy::{audio::Volume, ecs::system::SystemParam, prelude::*};

#[derive(Message, Clone, Copy)]
pub enum SoundCue {
    Step,
    Door,
    Pickup,
    Throw,
    Impact(Vec3),
    Alert,
    Caught,
    Win,
    Toggle,
    Ring,
    Read,
    NeighborStep(Vec3),
}
#[derive(Resource)]
struct Sounds(Vec<Handle<AudioSource>>);
#[derive(Component)]
struct GameAudio;
#[derive(Component)]
struct Ambience(LoopKind);
#[derive(Clone, Copy)]
enum LoopKind {
    Wind,
    Night,
    House,
    Underground,
    Generator,
    Windmill,
    Train,
    Prop(Entity, PropKind),
}
pub struct SoundPlugin;
impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(crate::music::MusicPlugin)
            .add_systems(Startup, setup_audio.after(GameSet::Setup))
            .add_systems(
                Update,
                (play_cues, mix_audio).chain().after(GameSet::Presentation),
            );
    }
}
fn setup_audio(
    mut commands: Commands,
    mut sources: ResMut<Assets<AudioSource>>,
    options: Res<LaunchOptions>,
    props: Query<(Entity, &InteractiveProp)>,
) {
    let sounds = [
        gen_step_wav(),
        gen_door_wav(),
        gen_key_wav(),
        gen_throw_wav(),
        gen_impact_wav(),
        gen_alert_wav(),
        gen_caught_wav(),
        gen_win_wav(),
        chime(0.1, 620.),
        ring(),
        chime(0.2, 340.),
    ]
    .into_iter()
    .map(|bytes| {
        sources.add(AudioSource {
            bytes: bytes.into(),
        })
    })
    .collect();
    commands.insert_resource(Sounds(sounds));
    if options.quiet {
        return;
    }
    for kind in [
        LoopKind::Wind,
        LoopKind::Night,
        LoopKind::House,
        LoopKind::Underground,
        LoopKind::Generator,
        LoopKind::Windmill,
        LoopKind::Train,
    ] {
        spawn_loop(&mut commands, &mut sources, kind, ambient_wav(kind));
    }
    // Reuse samples for taps and appliances rather than synthesizing per frame.
    for prop_kind in [
        PropKind::Television,
        PropKind::Telephone,
        PropKind::Sink,
        PropKind::Shower,
        PropKind::Kettle,
    ] {
        let sample = sources.add(AudioSource {
            bytes: ambient_wav(LoopKind::Prop(Entity::PLACEHOLDER, prop_kind)).into(),
        });
        for (id, prop) in &props {
            if prop.kind == prop_kind {
                commands.spawn((
                    Name::new("Household sound"),
                    GameAudio,
                    Ambience(LoopKind::Prop(id, prop_kind)),
                    AudioPlayer::new(sample.clone()),
                    PlaybackSettings::LOOP.with_volume(Volume::Linear(0.)),
                ));
            }
        }
    }
}
fn spawn_loop(
    commands: &mut Commands,
    sources: &mut Assets<AudioSource>,
    kind: LoopKind,
    bytes: Vec<u8>,
) {
    let source = sources.add(AudioSource {
        bytes: bytes.into(),
    });
    commands.spawn((
        Name::new("Environmental sound layer"),
        GameAudio,
        Ambience(kind),
        AudioPlayer::new(source),
        PlaybackSettings::LOOP.with_volume(Volume::Linear(0.)),
    ));
}
fn play_cues(
    mut commands: Commands,
    mut messages: MessageReader<SoundCue>,
    sounds: Res<Sounds>,
    options: Res<LaunchOptions>,
    players: Query<&Player>,
    collision: Res<CollisionWorld>,
) {
    for cue in messages.read() {
        if options.quiet {
            continue;
        }
        let (index, volume) = match cue {
            SoundCue::Step => (0, 0.2),
            SoundCue::Door => (1, 0.45),
            SoundCue::Pickup => (2, 0.4),
            SoundCue::Throw => (3, 0.4),
            SoundCue::Impact(pos) => (
                4,
                players.single().map_or(0., |p| {
                    distance_gain(p.eye(), *pos + Vec3::Y * 0.1, 22., &collision.solids)
                }) * 0.45,
            ),
            SoundCue::Alert => (5, 0.4),
            SoundCue::Caught => (6, 0.45),
            SoundCue::Win => (7, 0.4),
            SoundCue::Toggle => (8, 0.3),
            SoundCue::Ring => (9, 0.22),
            SoundCue::Read => (10, 0.22),
            SoundCue::NeighborStep(pos) => (
                0,
                players.single().map_or(0., |p| {
                    distance_gain(p.eye(), *pos + Vec3::Y, 17., &collision.solids)
                }) * 0.42,
            ),
        };
        let volume = volume * options.effects_volume;
        if volume < 0.001 {
            continue;
        }
        commands.spawn((
            GameAudio,
            AudioPlayer::new(sounds.0[index].clone()),
            PlaybackSettings::DESPAWN.with_volume(Volume::Linear(volume)),
        ));
    }
}
#[derive(SystemParam)]
struct MixState<'w> {
    options: Res<'w, LaunchOptions>,
    session: Res<'w, Session>,
    clock: Res<'w, WorldClock>,
    progress: Res<'w, PuzzleProgress>,
    basement: Res<'w, crate::basement::BasementProgress>,
    collision: Res<'w, CollisionWorld>,
    time: Res<'w, Time>,
}
fn mix_audio(
    state: MixState,
    players: Query<&Player>,
    props: Query<&InteractiveProp>,
    trains: Query<&Transform, With<Train>>,
    mut sinks: Query<(&mut AudioSink, Option<&Ambience>), With<GameAudio>>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let eye = player.eye();
    let underground = player.pos.y < -1.5;
    let indoors = underground
        || (-12.0..12.0).contains(&eye.x) && (-10.0..8.0).contains(&eye.z) && eye.y < 18.;
    let daylight =
        (((state.clock.hour - 6.) / 24. * std::f32::consts::TAU).sin() * 3.).clamp(0., 1.);
    for (mut sink, ambient) in &mut sinks {
        if state.session.paused {
            sink.pause();
            continue;
        }
        sink.play();
        let Some(ambient) = ambient else {
            continue;
        };
        let near = |pos: Vec3, range| distance_gain(eye, pos, range, &state.collision.solids);
        let gain = if state.session.outcome != Outcome::Exploring {
            0.
        } else {
            match ambient.0 {
                LoopKind::Wind => {
                    if underground {
                        0.
                    } else if indoors {
                        0.06
                    } else {
                        0.23
                    }
                }
                LoopKind::Night => {
                    (1. - daylight)
                        * if underground {
                            0.
                        } else if indoors {
                            0.025
                        } else {
                            0.15
                        }
                }
                LoopKind::Underground => {
                    if underground {
                        if state.basement.circuits[2] {
                            0.4
                        } else {
                            0.25
                        }
                    } else {
                        0.
                    }
                }
                LoopKind::House => {
                    if underground {
                        0.
                    } else if indoors {
                        0.12
                    } else {
                        0.015
                    }
                }
                LoopKind::Generator => {
                    if state.progress.power_off {
                        0.
                    } else {
                        near(GENERATOR + Vec3::Z, 18.) * 0.38
                    }
                }
                LoopKind::Windmill => {
                    if state.progress.windmill_stopped {
                        0.
                    } else {
                        near(WINDMILL_SWITCH + Vec3::Z, 23.) * 0.3
                    }
                }
                LoopKind::Train => {
                    if state.progress.train_running {
                        trains
                            .single()
                            .map_or(0., |t| near(t.translation + Vec3::Y, 25.) * 0.3)
                    } else {
                        0.
                    }
                }
                LoopKind::Prop(id, _) => props.get(id).map_or(0., |p| {
                    if p.active {
                        near(p.position + Vec3::Y * 0.15, 17.) * 0.32
                    } else {
                        0.
                    }
                }),
            }
        };
        let gain = gain * state.options.ambience_volume;
        let volume = sink.volume().to_linear();
        sink.set_volume(Volume::Linear(
            volume + (gain - volume) * (1. - (-6. * state.time.delta_secs()).exp()),
        ));
    }
}
fn distance_gain(eye: Vec3, source: Vec3, range: f32, solids: &[crate::world::Aabb]) -> f32 {
    let distance = eye.distance(source);
    if distance >= range {
        return 0.;
    }
    let gain = (1. - distance / range).clamp(0., 1.).powi(2);
    gain * if line_of_sight(eye, source, solids) {
        1.
    } else {
        0.3
    }
}
fn chime(seconds: f32, frequency: f32) -> Vec<u8> {
    let rate = 22050;
    let samples = (0..(seconds * rate as f32) as usize)
        .map(|i| {
            let t = i as f32 / rate as f32;
            ((t * frequency * std::f32::consts::TAU).sin() * (-t * 35.).exp() * 5000.) as i16
        })
        .collect::<Vec<_>>();
    create_wav(rate, &samples)
}
fn ring() -> Vec<u8> {
    let rate = 22050;
    let samples = (0..rate / 2)
        .map(|i| {
            let t = i as f32 / rate as f32;
            let envelope = (t * 100.).min(1.) * ((0.5 - t) * 60.).min(1.);
            let tone =
                (t * 620. * std::f32::consts::TAU).sin() + (t * 830. * std::f32::consts::TAU).sin();
            (tone * envelope * 3500.) as i16
        })
        .collect::<Vec<_>>();
    create_wav(rate, &samples)
}
fn ambient_wav(kind: LoopKind) -> Vec<u8> {
    let rate = 22050;
    let mut rng = 87234;
    let mut filtered = 0.;
    let samples = (0..rate * 8)
        .map(|i| {
            let t = i as f32 / rate as f32;
            let phase = t * std::f32::consts::TAU;
            filtered += (lcg_noise(&mut rng) - filtered) * 0.045;
            let tone = match kind {
                LoopKind::Wind => filtered * (0.4 + 0.2 * (phase / 8.).sin()),
                LoopKind::Night => {
                    (phase * 2700.).sin()
                        * (phase * 18.).sin().max(0.).powi(8)
                        * (phase / 2.).sin().max(0.)
                        * 0.08
                }
                LoopKind::House => (phase * 55.).sin() * 0.035 + (phase * 82.5).sin() * 0.02,
                LoopKind::Underground => {
                    let drip = t.rem_euclid(1.6);
                    let echo =
                        (drip * 780. * std::f32::consts::TAU).sin() * (-drip * 19.).exp() * 0.15;
                    (phase * 41.25).sin() * 0.04 + filtered * 0.15 + echo
                }
                LoopKind::Generator => {
                    ((phase * 70.).sin() + (phase * 140.).sin() * 0.35)
                        * (0.1 + 0.02 * (phase * 8.).sin())
                }
                LoopKind::Windmill => {
                    filtered * (phase * 1.5).sin().max(0.) * 0.6 + (phase * 110.).sin() * 0.02
                }
                LoopKind::Train => {
                    filtered * 0.5 + (phase * 90.).sin() * (phase * 6.).sin().max(0.) * 0.1
                }
                LoopKind::Prop(_, PropKind::Telephone) => {
                    let gate = if t.rem_euclid(2.) < 0.6 { 1. } else { 0. };
                    ((phase * 620.).sin() + (phase * 830.).sin()) * 0.08 * gate
                }
                LoopKind::Prop(_, PropKind::Television) => {
                    lcg_noise(&mut rng) * 0.07 + (phase * 60.).sin() * 0.015
                }
                LoopKind::Prop(_, PropKind::Kettle) => {
                    filtered * 0.25 + (phase * 1800.).sin() * 0.025
                }
                LoopKind::Prop(_, _) => filtered * 0.7 + lcg_noise(&mut rng) * 0.03,
            };
            // Short edge ramps suppress clicks when looping deterministic generated buffers.
            let envelope = (t * 50.).min(1.) * ((8. - t) * 50.).min(1.);
            (tone * envelope * 32767.).clamp(-32767., 32767.) as i16
        })
        .collect::<Vec<_>>();
    create_wav(rate, &samples)
}

fn create_wav(sample_rate: u32, samples: &[i16]) -> Vec<u8> {
    pcm_wav(sample_rate, 1, samples)
}

pub(crate) fn pcm_wav(sample_rate: u32, channels: u16, samples: &[i16]) -> Vec<u8> {
    let mut out = Vec::with_capacity(44 + samples.len() * 2);
    let data_len = (samples.len() * 2) as u32;
    let file_len = 36 + data_len;

    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&file_len.to_le_bytes());
    out.extend_from_slice(b"WAVE");
    out.extend_from_slice(b"fmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * u32::from(channels) * 2;
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&(channels * 2).to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());
    for &sample in samples {
        out.extend_from_slice(&sample.to_le_bytes());
    }
    out
}

fn lcg_noise(state: &mut u32) -> f32 {
    *state = state.wrapping_mul(1664525).wrapping_add(1013904223);
    ((*state >> 16) as f32 / 32767.5) - 1.0
}

fn gen_step_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.06) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 12345u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (1.0 - t / 0.06).max(0.0).powi(2);
        let freq = 90.0 - t * 400.0;
        let tone = (2.0 * std::f32::consts::PI * freq * t).sin();
        let noise = lcg_noise(&mut rng) * 0.25;
        let val = (tone * 0.75 + noise) * env * 0.45;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_door_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.35) as usize;
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = if t < 0.05 {
            t / 0.05
        } else {
            (1.0 - (t - 0.05) / 0.3).max(0.0)
        };
        let wobble = (2.0 * std::f32::consts::PI * 8.0 * t).sin() * 15.0;
        let freq = 130.0 + t * 90.0 + wobble;
        let phase = 2.0 * std::f32::consts::PI * freq * t;
        let tone = phase.sin() * 0.6 + (phase * 2.0).sin() * 0.3;
        let val = tone * env * 0.5;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_key_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.35) as usize;
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env1 = (-t * 12.0).exp();
        let env2 = if t > 0.08 {
            (-(t - 0.08) * 10.0).exp()
        } else {
            0.0
        };
        let tone1 = (2.0 * std::f32::consts::PI * 1318.5 * t).sin() * env1;
        let tone2 = (2.0 * std::f32::consts::PI * 1975.5 * t).sin() * env2;
        let val = (tone1 * 0.5 + tone2 * 0.5) * 0.6;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_throw_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.16) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 54321u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (-(t - 0.06).powi(2) / 0.003).exp();
        let freq = 280.0 - t * 800.0;
        let tone = (2.0 * std::f32::consts::PI * freq.max(50.0) * t).sin() * 0.4;
        let noise = lcg_noise(&mut rng) * 0.6;
        let val = (tone + noise) * env * 0.5;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_impact_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.22) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 98765u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (-t * 18.0).exp();
        let freq = 85.0 - t * 200.0;
        let tone = (2.0 * std::f32::consts::PI * freq.max(35.0) * t).sin();
        let click = if t < 0.015 {
            lcg_noise(&mut rng) * 0.5
        } else {
            0.0
        };
        let val = (tone * 0.7 + click) * env * 0.65;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_alert_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.5) as usize;
    let mut samples = Vec::with_capacity(count);
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = if t < 0.04 {
            t / 0.04
        } else {
            (1.0 - (t - 0.04) / 0.46).max(0.0)
        };
        let tone1 = (2.0 * std::f32::consts::PI * 220.0 * t).sin();
        let tone2 = (2.0 * std::f32::consts::PI * 311.1 * t).sin();
        let tone3 = (2.0 * std::f32::consts::PI * 622.2 * t).sin();
        let val = (tone1 * 0.4 + tone2 * 0.4 + tone3 * 0.2) * env * 0.55;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_caught_wav() -> Vec<u8> {
    let rate = 22050u32;
    let count = (rate as f32 * 0.6) as usize;
    let mut samples = Vec::with_capacity(count);
    let mut rng = 13579u32;
    for i in 0..count {
        let t = i as f32 / rate as f32;
        let env = (-t * 6.0).exp();
        let freq = 70.0 - t * 40.0;
        let tone = (2.0 * std::f32::consts::PI * freq.max(30.0) * t).sin();
        let noise = if t < 0.12 {
            lcg_noise(&mut rng) * (1.0 - t / 0.12) * 0.6
        } else {
            0.0
        };
        let val = (tone * 0.65 + noise) * env * 0.7;
        samples.push((val.clamp(-1.0, 1.0) * 32767.0) as i16);
    }
    create_wav(rate, &samples)
}

fn gen_win_wav() -> Vec<u8> {
    drone(1.8, 98.0, 0.3)
}
fn drone(seconds: f32, base: f32, gain: f32) -> Vec<u8> {
    let rate = 22050;
    let samples = (0..(seconds * rate as f32) as usize)
        .map(|i| {
            let t = i as f32 / rate as f32;
            let fade = (t * 2.).min(1.) * ((seconds - t) * 2.).min(1.);
            let pulse = 0.65 + 0.35 * (t * 0.7).sin();
            let tone = (t * base * std::f32::consts::TAU).sin() * 0.55
                + (t * base * 1.007 * std::f32::consts::TAU).sin() * 0.3
                + (t * base * 1.498 * std::f32::consts::TAU).sin() * 0.15;
            (tone * gain * fade * pulse * 32767.) as i16
        })
        .collect::<Vec<_>>();
    create_wav(rate, &samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn generated_background_layers_are_valid_bounded_non_silent_pcm() {
        for kind in [
            LoopKind::Wind,
            LoopKind::Night,
            LoopKind::House,
            LoopKind::Underground,
            LoopKind::Generator,
            LoopKind::Windmill,
            LoopKind::Train,
            LoopKind::Prop(Entity::PLACEHOLDER, PropKind::Television),
            LoopKind::Prop(Entity::PLACEHOLDER, PropKind::Telephone),
            LoopKind::Prop(Entity::PLACEHOLDER, PropKind::Sink),
            LoopKind::Prop(Entity::PLACEHOLDER, PropKind::Shower),
            LoopKind::Prop(Entity::PLACEHOLDER, PropKind::Kettle),
        ] {
            let wav = ambient_wav(kind);
            assert_eq!(&wav[..4], b"RIFF");
            assert_eq!(&wav[8..12], b"WAVE");
            assert_eq!(wav.len(), 44 + 22050 * 8 * 2);
            let peak = wav[44..]
                .as_chunks::<2>()
                .0
                .iter()
                .map(|x| i16::from_le_bytes([x[0], x[1]]).unsigned_abs())
                .max()
                .unwrap();
            assert!(
                peak > 100 && peak < 30000,
                "silent or clipping loop: peak {peak}"
            );
            assert_eq!(&wav[44..46], &[0, 0]);
        }
    }
    #[test]
    fn source_distance_and_walls_reduce_sound_without_leaking_past_range() {
        let source = Vec3::new(0., 1., 0.);
        let near = Vec3::new(0., 1., 2.);
        let wall = crate::world::Aabb::from_center(Vec3::new(0., 1., 1.), Vec3::new(5., 4., 0.2));
        assert!(distance_gain(near, source, 10., &[wall]) < distance_gain(near, source, 10., &[]));
        assert!(
            distance_gain(near, source, 10., &[])
                > distance_gain(Vec3::new(0., 1., 8.), source, 10., &[])
        );
        assert_eq!(distance_gain(Vec3::X * 30., source, 10., &[]), 0.);
    }
}
