//! An original adaptive score, independent of the world's ambient sound sources.
//!
//! Two finite decks crossfade between authored synthesized themes. A short dwell
//! avoids score changes at every doorway; danger overrides the dwell. Each theme
//! has two musical arrangements, alternating before the current phrase ends.
use crate::{
    LaunchOptions,
    ai::{AlertState, Neighbor},
    atmosphere::WorldClock,
    audio::{SoundCue, pcm_wav},
    game::{GameSet, Outcome, Session},
    items::PuzzleProgress,
    player::Player,
    world::{GENERATOR, TRAIN_SWITCH, WINDMILL_SWITCH},
};
use bevy::{audio::Volume, ecs::system::SystemParam, prelude::*};
use std::f32::consts::{FRAC_PI_4, TAU};

const SAMPLE_RATE: u32 = 22050;
const BEAT: f32 = 0.75; // 80 BPM; danger doubles the subdivision, not playback pitch.
const BARS: usize = 8;
const TRACK_SECONDS: f32 = BEAT * 4. * BARS as f32;
const ARRANGEMENTS: usize = 2;
const CALM_FADE: f32 = 2.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
enum Theme {
    Morning,
    Afternoon,
    Evening,
    Night,
    House,
    Machinery,
    PowerDown,
    Underground,
    Suspicion,
    Chase,
    Search,
    Caught,
    Completion,
}
impl Theme {
    const ALL: [Self; 13] = [
        Self::Morning,
        Self::Afternoon,
        Self::Evening,
        Self::Night,
        Self::House,
        Self::Machinery,
        Self::PowerDown,
        Self::Underground,
        Self::Suspicion,
        Self::Chase,
        Self::Search,
        Self::Caught,
        Self::Completion,
    ];
    fn label(self) -> &'static str {
        match self {
            Self::Morning => "Morning on Friendly Court",
            Self::Afternoon => "The crooked afternoon",
            Self::Evening => "Long shadows",
            Self::Night => "After the streetlights",
            Self::House => "Someone lives here",
            Self::Machinery => "The house is a machine",
            Self::PowerDown => "When the power stops",
            Self::Suspicion => "Did he hear that?",
            Self::Chase => "Run from the neighbor",
            Self::Search => "Keep out of sight",
            Self::Caught => "Another failed attempt",
            Self::Completion => "Beyond the service door",
            Self::Underground => "Beneath the impossible house",
        }
    }
    fn urgent(self) -> bool {
        matches!(self, Self::Chase | Self::Caught | Self::Completion)
    }
    fn fade_seconds(self) -> f32 {
        match self {
            Self::Chase => 0.45,
            Self::Caught | Self::Completion => 0.85,
            Self::Suspicion | Self::Search => 1.3,
            _ => CALM_FADE,
        }
    }
    fn gain(self) -> f32 {
        match self {
            Self::Chase => 0.68,
            Self::Suspicion | Self::Search => 0.58,
            Self::Night | Self::PowerDown | Self::Caught => 0.48,
            _ => 0.55,
        }
    }
}

fn choose_theme(
    session: &Session,
    clock: &WorldClock,
    progress: &PuzzleProgress,
    player: &Player,
    neighbor: Option<&Neighbor>,
) -> Theme {
    match session.outcome {
        Outcome::Caught => return Theme::Caught,
        Outcome::Escaped => return Theme::Completion,
        Outcome::Exploring => {}
    }
    // A distant distraction must not reveal the neighbor's state through walls.
    // Across the safe street, the music returns to the neighborhood theme.
    if player.pos.z < 17.
        && let Some(n) = neighbor
    {
        if n.state == AlertState::Chase {
            return Theme::Chase;
        }
        if player.pos.distance(n.body.pos) < 23. {
            match n.state {
                AlertState::Search => return Theme::Search,
                AlertState::Investigate => return Theme::Suspicion,
                _ if n.awareness > 0.18 => return Theme::Suspicion,
                _ => {}
            }
        }
    }
    if player.pos.y < -1.5 {
        return Theme::Underground;
    }
    let eye = player.eye();
    let indoors = (-12.0..12.0).contains(&eye.x) && (-10.0..8.0).contains(&eye.z) && eye.y < 18.;
    if (eye.distance(GENERATOR) < 5.5 && !progress.power_off)
        || (eye.distance(WINDMILL_SWITCH) < 5.5 && !progress.windmill_stopped)
        || (eye.distance(TRAIN_SWITCH) < 5.5 && progress.train_running)
    {
        return Theme::Machinery;
    }
    if indoors {
        return if progress.power_off {
            Theme::PowerDown
        } else {
            Theme::House
        };
    }
    match clock.phase() {
        "MORNING" => Theme::Morning,
        "AFTERNOON" => Theme::Afternoon,
        "EVENING" => Theme::Evening,
        _ => Theme::Night,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Track {
    theme: Theme,
    arrangement: usize,
}
#[derive(Clone, Copy, Debug)]
struct Deck {
    track: Track,
    age: f32,
    weight: f32,
}
#[derive(Clone, Copy)]
struct Fade {
    from: Option<usize>,
    to: usize,
    amount: f32,
    seconds: f32,
}
#[derive(Resource)]
struct MusicDirector {
    decks: [Option<Deck>; 2],
    active: usize,
    fade: Option<Fade>,
    candidate: Theme,
    candidate_for: f32,
    held_for: f32,
    attempt: u32,
    enabled: bool,
    duck_for: f32,
    duck_gain: f32,
}
impl Default for MusicDirector {
    fn default() -> Self {
        Self {
            decks: [None, None],
            active: 0,
            fade: None,
            candidate: Theme::Evening,
            candidate_for: 0.,
            held_for: 0.,
            attempt: 0,
            enabled: true,
            duck_for: 0.,
            duck_gain: 1.,
        }
    }
}
impl MusicDirector {
    fn reset_attempt(&mut self, attempt: u32) {
        *self = Self {
            attempt,
            enabled: self.enabled,
            ..default()
        };
    }
    fn duck(&mut self, cue: SoundCue) {
        let seconds: f32 = match cue {
            SoundCue::Caught | SoundCue::Win | SoundCue::Alert => 1.2,
            SoundCue::Pickup | SoundCue::Read => 0.55,
            _ => 0.,
        };
        self.duck_for = self.duck_for.max(seconds);
    }
    fn start(&mut self, track: Track) {
        let from = self.decks[self.active].map(|_| self.active);
        let to = if from.is_some() { 1 - self.active } else { 0 };
        self.decks[to] = Some(Deck {
            track,
            age: 0.,
            weight: 0.,
        });
        self.active = to;
        self.held_for = 0.;
        self.fade = Some(Fade {
            from,
            to,
            amount: 0.,
            seconds: track.theme.fade_seconds(),
        });
    }
    fn advance(&mut self, theme: Theme, dt: f32, paused: bool) {
        if paused {
            return;
        }
        self.duck_for = (self.duck_for - dt).max(0.);
        let target = if self.duck_for > 0. { 0.35 } else { 1. };
        let rate = if target < self.duck_gain { 14. } else { 2.8 };
        self.duck_gain += (target - self.duck_gain) * (1. - (-rate * dt).exp());
        if theme != self.candidate {
            self.candidate = theme;
            self.candidate_for = 0.;
        }
        self.candidate_for += dt;
        self.held_for += dt;
        for deck in self.decks.iter_mut().flatten() {
            deck.age += dt;
        }
        let Some(active) = self.decks[self.active] else {
            self.start(Track {
                theme,
                arrangement: 0,
            });
            return;
        };
        if let Some(mut fade) = self.fade {
            // Finish an in-flight crossfade rapidly before accepting new danger.
            // This preserves continuity without allocating a third audio voice.
            if theme.urgent() && active.track.theme != theme {
                fade.seconds = fade.seconds.min(0.12);
            }
            fade.amount = (fade.amount + dt / fade.seconds).min(1.);
            if let Some(from) = fade.from {
                self.decks[from].as_mut().unwrap().weight = 1. - fade.amount;
            }
            self.decks[fade.to].as_mut().unwrap().weight = fade.amount;
            if fade.amount >= 1. {
                if let Some(from) = fade.from {
                    self.decks[from] = None;
                }
                self.fade = None;
            } else {
                self.fade = Some(fade);
            }
        }
        if self.fade.is_some() {
            return;
        }
        let change = theme != active.track.theme
            && (theme.urgent() || (self.candidate_for >= 1.1 && self.held_for >= 3.5));
        if change {
            self.start(Track {
                theme,
                arrangement: 0,
            });
        } else if active.age >= TRACK_SECONDS - CALM_FADE - 0.5 {
            self.start(Track {
                theme: active.track.theme,
                arrangement: (active.track.arrangement + 1) % ARRANGEMENTS,
            });
        }
    }
    fn volume(&self, deck: Deck, user_volume: f32) -> f32 {
        if self.enabled {
            deck.weight * deck.track.theme.gain() * self.duck_gain * user_volume
        } else {
            0.
        }
    }
}

#[derive(Resource)]
struct MusicBank(Vec<[Handle<AudioSource>; ARRANGEMENTS]>);
#[derive(Component)]
struct MusicDeck {
    slot: usize,
    track: Track,
    attempt: u32,
}

pub(crate) struct MusicPlugin;
impl Plugin for MusicPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MusicDirector>()
            .add_systems(Startup, setup_music.after(GameSet::Setup))
            .add_systems(
                Update,
                (direct_music, mix_music)
                    .chain()
                    .after(GameSet::Presentation),
            );
    }
}
fn setup_music(
    mut commands: Commands,
    options: Res<LaunchOptions>,
    mut sources: ResMut<Assets<AudioSource>>,
) {
    if options.quiet {
        return;
    }
    let tracks = Theme::ALL
        .into_iter()
        .map(|theme| {
            std::array::from_fn(|arrangement| {
                sources.add(AudioSource {
                    bytes: compose(Track { theme, arrangement }).into(),
                })
            })
        })
        .collect();
    commands.insert_resource(MusicBank(tracks));
}
#[derive(SystemParam)]
struct ScoreWorld<'w, 's> {
    options: Res<'w, LaunchOptions>,
    time: Res<'w, Time>,
    session: ResMut<'w, Session>,
    clock: Res<'w, WorldClock>,
    progress: Res<'w, PuzzleProgress>,
    keys: Res<'w, ButtonInput<KeyCode>>,
    players: Query<'w, 's, &'static Player>,
    neighbors: Query<'w, 's, &'static Neighbor>,
}
fn direct_music(
    mut commands: Commands,
    mut state: ScoreWorld,
    mut director: ResMut<MusicDirector>,
    bank: Option<Res<MusicBank>>,
    mut cues: MessageReader<SoundCue>,
    decks: Query<(Entity, &MusicDeck)>,
) {
    if director.attempt != state.session.attempts {
        director.reset_attempt(state.session.attempts);
    }
    for cue in cues.read() {
        director.duck(*cue);
    }
    if state.keys.just_pressed(KeyCode::KeyM) {
        if state.options.quiet {
            state.session.tell("Audio is disabled by --quiet.");
        } else {
            director.enabled = !director.enabled;
            state.session.tell(if director.enabled {
                "Background music on. M to mute music."
            } else {
                "Background music off. M to restore music."
            });
        }
    }
    let Some(bank) = bank else {
        return;
    };
    if state.options.quiet {
        return;
    }
    let Ok(player) = state.players.single() else {
        return;
    };
    let theme = choose_theme(
        &state.session,
        &state.clock,
        &state.progress,
        player,
        state.neighbors.single().ok(),
    );
    director.advance(theme, state.time.delta_secs(), state.session.paused);
    let mut present = [false; 2];
    for (id, playing) in &decks {
        let keep = playing.attempt == director.attempt
            && director.decks[playing.slot].is_some_and(|d| d.track == playing.track);
        if keep {
            present[playing.slot] = true;
        } else {
            commands.entity(id).despawn();
        }
    }
    for (slot, deck) in director.decks.iter().enumerate() {
        if let Some(deck) = deck
            && !present[slot]
        {
            debug!(
                "Score: {} / arrangement {}",
                deck.track.theme.label(),
                deck.track.arrangement + 1
            );
            let mut settings = PlaybackSettings::ONCE.with_volume(Volume::Linear(0.));
            settings.paused = state.session.paused;
            commands.spawn((
                Name::new(format!("Score · {}", deck.track.theme.label())),
                MusicDeck {
                    slot,
                    track: deck.track,
                    attempt: director.attempt,
                },
                AudioPlayer::new(bank.0[deck.track.theme as usize][deck.track.arrangement].clone()),
                settings,
            ));
        }
    }
}
fn mix_music(
    options: Res<LaunchOptions>,
    session: Res<Session>,
    director: Res<MusicDirector>,
    mut sinks: Query<(&MusicDeck, &mut AudioSink)>,
) {
    for (playing, mut sink) in &mut sinks {
        if session.paused {
            sink.pause();
        } else {
            sink.play();
        }
        let gain = director.decks[playing.slot].map_or(0., |deck| {
            director.volume(deck, options.music_volume) * f32::from(!options.quiet)
        });
        sink.set_volume(Volume::Linear(gain));
    }
}

#[derive(Clone, Copy)]
enum Instrument {
    Felt,
    Bell,
    Pluck,
    Pad,
    Bass,
    Tick,
    Drum,
}
fn note(
    buffer: &mut [[f32; 2]],
    start: f32,
    length: f32,
    midi: f32,
    instrument: Instrument,
    gain: f32,
    pan: f32,
) {
    let begin = (start * SAMPLE_RATE as f32).round() as usize;
    let count = (length * SAMPLE_RATE as f32).round() as usize;
    let frequency = 440. * 2_f32.powf((midi - 69.) / 12.);
    let angle = (pan.clamp(-1., 1.) + 1.) * FRAC_PI_4;
    let (right, left) = angle.sin_cos();
    for (i, frame) in buffer.iter_mut().skip(begin).take(count).enumerate() {
        let t = i as f32 / SAMPLE_RATE as f32;
        let phase = TAU * frequency * t;
        let attack = if matches!(instrument, Instrument::Pad) {
            (t / 0.25).min(1.)
        } else {
            (t / 0.008).min(1.)
        };
        let release = ((length - t) / 0.12).clamp(0., 1.);
        let tone = match instrument {
            Instrument::Felt => {
                (phase.sin() * 0.78 + (phase * 2.003).sin() * 0.17 + (phase * 3.99).sin() * 0.05)
                    * (-t * 3.4).exp()
            }
            Instrument::Bell => {
                (phase.sin() * 0.72 + (phase * 2.76).sin() * 0.19 + (phase * 4.08).sin() * 0.09)
                    * (-t * 2.4).exp()
            }
            Instrument::Pluck => {
                (phase.sin() * 0.72 + (phase * 2.).sin() * 0.2 + (phase * 3.).sin() * 0.08)
                    * (-t * 7.).exp()
            }
            Instrument::Pad => {
                (phase.sin() * 0.6 + (phase * 1.002).sin() * 0.27 + (phase * 2.).sin() * 0.13)
                    * (0.9 + 0.1 * (TAU * 0.7 * t).sin())
            }
            Instrument::Bass => (phase.sin() * 0.85 + (phase * 2.).sin() * 0.15) * (-t * 2.).exp(),
            Instrument::Tick => {
                ((phase * 1.41).sin() * 0.6 + (phase * 2.17).sin() * 0.4) * (-t * 55.).exp()
            }
            Instrument::Drum => {
                // Integrated downward pitch sweep avoids phase discontinuities.
                let phase = TAU * (45. * t + 3. * (1. - (-t * 22.).exp()));
                phase.sin() * (-t * 13.).exp()
            }
        };
        let sample = tone * attack * release * gain;
        frame[0] += sample * left;
        frame[1] += sample * right;
    }
}

fn compose(track: Track) -> Vec<u8> {
    let mut buffer = vec![[0., 0.]; (TRACK_SECONDS * SAMPLE_RATE as f32) as usize];
    let theme = track.theme;
    let urgent = matches!(theme, Theme::Chase | Theme::Suspicion | Theme::Search);
    let roots = if theme == Theme::Completion {
        [50., 46., 48., 50.]
    } else {
        [50., 46., 43., 45.]
    };
    let motif = match theme {
        Theme::Morning => [74, 0, 77, 0, 81, 79, 77, 76],
        Theme::Afternoon => [74, 81, 0, 77, 76, 0, 72, 0],
        Theme::Evening => [69, 0, 74, 0, 77, 0, 76, 0],
        Theme::Night => [86, 0, 0, 81, 0, 0, 77, 0],
        Theme::House => [74, 0, 75, 0, 69, 0, 77, 76],
        Theme::Machinery => [62, 69, 65, 69, 62, 68, 65, 69],
        Theme::PowerDown => [62, 0, 0, 0, 69, 0, 0, 65],
        Theme::Underground => [50, 0, 57, 0, 51, 0, 0, 56],
        Theme::Suspicion => [62, 0, 63, 0, 69, 0, 68, 0],
        Theme::Chase => [62, 69, 63, 69, 65, 69, 68, 69],
        Theme::Search => [74, 0, 0, 75, 0, 69, 0, 0],
        Theme::Caught => [65, 0, 62, 0, 57, 0, 0, 0],
        Theme::Completion => [74, 0, 77, 79, 81, 0, 78, 86],
    };
    let instrument = match theme {
        Theme::Morning | Theme::Night | Theme::Completion => Instrument::Bell,
        Theme::Afternoon | Theme::Machinery | Theme::Chase => Instrument::Pluck,
        _ => Instrument::Felt,
    };
    for bar in 0..BARS {
        let root = roots[(bar / 2 + track.arrangement) % roots.len()];
        let third = if root == 46. || root == 45. || theme == Theme::Completion {
            4.
        } else {
            3.
        };
        let pad_gain = if urgent { 0.028 } else { 0.037 };
        for (part, offset) in [0., third, 7.].into_iter().enumerate() {
            note(
                &mut buffer,
                bar as f32 * BEAT * 4.,
                BEAT * 4.,
                root + offset + 12.,
                Instrument::Pad,
                pad_gain,
                (part as f32 - 1.) * 0.55,
            );
        }
        for step in 0..8 {
            let at = bar as f32 * BEAT * 4. + step as f32 * BEAT * 0.5;
            let index = (step + track.arrangement * 3 + (bar % 2) * 2) % motif.len();
            let pitch = motif[index];
            if pitch != 0 && (theme != Theme::PowerDown || bar % 2 == 0) {
                let octave = if track.arrangement == 1 && bar >= 4 {
                    12.
                } else {
                    0.
                };
                note(
                    &mut buffer,
                    at,
                    if urgent { BEAT * 0.48 } else { BEAT * 1.45 },
                    pitch as f32 + octave,
                    instrument,
                    if urgent { 0.13 } else { 0.115 },
                    if step % 2 == 0 { -0.28 } else { 0.28 },
                );
            }
            if step % 4 == 0 || (theme == Theme::Chase && step % 2 == 0) {
                note(
                    &mut buffer,
                    at,
                    BEAT * 1.5,
                    root - 12.,
                    Instrument::Bass,
                    if theme == Theme::Chase { 0.15 } else { 0.08 },
                    0.,
                );
            }
            if matches!(
                theme,
                Theme::Machinery | Theme::Suspicion | Theme::Search | Theme::Chase
            ) && (step % 2 == 0 || theme == Theme::Chase)
            {
                note(
                    &mut buffer,
                    at,
                    0.11,
                    79. + (bar % 3) as f32 * 2.,
                    Instrument::Tick,
                    if theme == Theme::Chase { 0.085 } else { 0.045 },
                    if step % 4 == 0 { -0.6 } else { 0.6 },
                );
            }
            if theme == Theme::Chase && step % 2 == 0 {
                note(&mut buffer, at, 0.32, 40., Instrument::Drum, 0.18, 0.);
            }
        }
    }
    let last = buffer.len() - 1;
    let samples = buffer
        .into_iter()
        .enumerate()
        .flat_map(|(i, frame)| {
            let edge = (i as f32 / (SAMPLE_RATE as f32 * 0.012))
                .min((last - i) as f32 / (SAMPLE_RATE as f32 * 0.012))
                .min(1.);
            frame.map(|value| {
                // Soft saturation bounds overlap peaks without flattening attacks.
                (value / (1. + value.abs()) * edge * 0.92 * i16::MAX as f32) as i16
            })
        })
        .collect::<Vec<_>>();
    pcm_wav(SAMPLE_RATE, 2, &samples)
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::audio::{Decodable, Source};

    fn tick(director: &mut MusicDirector, theme: Theme, seconds: f32) {
        for _ in 0..(seconds * 60.).round() as usize {
            director.advance(theme, 1. / 60., false);
        }
    }
    fn active(director: &MusicDirector) -> Track {
        director.decks[director.active].unwrap().track
    }
    #[test]
    fn themes_follow_day_location_machinery_threat_and_outcome_in_priority_order() {
        let mut session = Session::default();
        let mut clock = WorldClock::default();
        let mut progress = PuzzleProgress::default();
        let mut player = Player::new(Vec3::new(0., 0., 20.));
        let mut neighbor = Neighbor::default();
        for (hour, expected) in [
            (9., Theme::Morning),
            (14., Theme::Afternoon),
            (17., Theme::Evening),
            (23., Theme::Night),
        ] {
            clock.hour = hour;
            assert_eq!(
                choose_theme(&session, &clock, &progress, &player, Some(&neighbor)),
                expected
            );
        }
        player.pos = Vec3::new(3., 0.2, 4.);
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, None),
            Theme::House
        );
        progress.power_off = true;
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, None),
            Theme::PowerDown
        );
        progress.power_off = false;
        for location in [GENERATOR, WINDMILL_SWITCH] {
            player.pos = location - Vec3::Y;
            assert_eq!(
                choose_theme(&session, &clock, &progress, &player, None),
                Theme::Machinery
            );
        }
        progress.train_running = true;
        player.pos = TRAIN_SWITCH - Vec3::Y;
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, None),
            Theme::Machinery
        );
        player.pos = crate::basement::ROOMS[0].center();
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, None),
            Theme::Underground
        );
        neighbor.body.pos = player.pos + Vec3::X * 5.;
        for (state, expected) in [
            (AlertState::Investigate, Theme::Suspicion),
            (AlertState::Chase, Theme::Chase),
            (AlertState::Search, Theme::Search),
        ] {
            neighbor.state = state;
            assert_eq!(
                choose_theme(&session, &clock, &progress, &player, Some(&neighbor)),
                expected
            );
        }
        session.outcome = Outcome::Caught;
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, Some(&neighbor)),
            Theme::Caught
        );
        session.outcome = Outcome::Escaped;
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, Some(&neighbor)),
            Theme::Completion
        );
    }
    #[test]
    fn distant_distractions_and_safe_street_do_not_trigger_threat_music() {
        let session = Session::default();
        let clock = WorldClock::default();
        let progress = PuzzleProgress::default();
        let mut player = Player::new(Vec3::new(3., 0., 4.));
        let mut neighbor = Neighbor::default();
        neighbor.state = AlertState::Investigate;
        neighbor.body.pos = player.pos + Vec3::X * 50.;
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, Some(&neighbor)),
            Theme::House
        );
        player.pos.z = 20.;
        neighbor.body.pos = player.pos;
        neighbor.state = AlertState::Chase;
        assert_eq!(
            choose_theme(&session, &clock, &progress, &player, Some(&neighbor)),
            Theme::Evening
        );
    }
    #[test]
    fn doorway_flicker_is_ignored_but_danger_interrupts_without_a_hard_cut() {
        let mut director = MusicDirector::default();
        tick(&mut director, Theme::Morning, 4.);
        for _ in 0..10 {
            tick(&mut director, Theme::House, 0.3);
            tick(&mut director, Theme::Morning, 0.3);
        }
        assert_eq!(active(&director).theme, Theme::Morning);
        tick(&mut director, Theme::House, 1.2);
        assert_eq!(active(&director).theme, Theme::House);
        let weights = director.decks.map(|d| d.map_or(0., |d| d.weight));
        director.advance(Theme::Chase, 1. / 60., false);
        let after = director.decks.map(|d| d.map_or(0., |d| d.weight));
        assert!(
            weights
                .iter()
                .zip(after)
                .all(|(a, b)| (a - b).abs() <= 0.15)
        );
        tick(&mut director, Theme::Chase, 0.18);
        assert_eq!(active(&director).theme, Theme::Chase);
        tick(&mut director, Theme::Chase, 0.6);
        assert!(director.fade.is_none());
        assert_eq!(director.decks.iter().flatten().count(), 1);
    }
    #[test]
    fn long_play_alternates_arrangements_with_two_bounded_crossfade_decks() {
        let mut director = MusicDirector::default();
        let mut changes = 0;
        let mut previous = None;
        for _ in 0..60 * 180 {
            director.advance(Theme::Night, 1. / 60., false);
            let track = active(&director);
            if previous != Some(track) {
                changes += 1;
                previous = Some(track);
            }
            let weights = director
                .decks
                .iter()
                .flatten()
                .map(|d| d.weight)
                .sum::<f32>();
            assert!((0.0..=1.0001).contains(&weights));
            for deck in director.decks.iter().flatten() {
                assert!(deck.age < TRACK_SECONDS, "track ended before crossfade");
            }
        }
        assert!(changes >= 8);
    }
    #[test]
    fn pause_retry_mute_and_cue_ducking_preserve_transport_and_user_choice() {
        let mut director = MusicDirector::default();
        director.reset_attempt(1);
        tick(&mut director, Theme::House, 5.);
        let before = director.decks;
        director.advance(Theme::Chase, 9., true);
        for (a, b) in before.into_iter().zip(director.decks) {
            assert_eq!(
                a.map(|d| (d.track, d.age, d.weight)),
                b.map(|d| (d.track, d.age, d.weight))
            );
        }
        let deck = director.decks[director.active].unwrap();
        let normal = director.volume(deck, 0.7);
        director.duck(SoundCue::Alert);
        tick(&mut director, Theme::House, 0.2);
        assert!(director.volume(deck, 0.7) < normal * 0.5);
        tick(&mut director, Theme::House, 3.);
        assert!(director.volume(deck, 0.7) > normal * 0.98);
        director.enabled = false;
        assert_eq!(director.volume(deck, 0.7), 0.);
        director.reset_attempt(2);
        assert!(director.decks.iter().all(Option::is_none));
        assert!(!director.enabled);
        tick(&mut director, Theme::Morning, 0.1);
        assert_eq!(active(&director).theme, Theme::Morning);
    }
    #[test]
    fn real_ecs_decks_switch_pause_mute_and_reset_without_accumulating_players() {
        let mut app = App::new();
        // Dummy handles exercise scheduling and entity cleanup without a sound device.
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f32(1. / 60.),
            ))
            .init_resource::<LaunchOptions>()
            .init_resource::<Session>()
            .init_resource::<WorldClock>()
            .init_resource::<PuzzleProgress>()
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<MusicDirector>()
            .insert_resource(MusicBank(
                Theme::ALL
                    .iter()
                    .map(|_| std::array::from_fn(|_| Handle::default()))
                    .collect(),
            ))
            .add_message::<SoundCue>()
            .add_systems(Update, direct_music);
        let player = app
            .world_mut()
            .spawn(Player::new(Vec3::new(0., 0., 20.)))
            .id();
        let neighbor = app.world_mut().spawn(Neighbor::default()).id();
        for _ in 0..240 {
            app.update();
        }
        let age = app.world().resource::<MusicDirector>().decks[0]
            .unwrap()
            .age;
        app.world_mut().resource_mut::<Session>().paused = true;
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(
            app.world().resource::<MusicDirector>().decks[0]
                .unwrap()
                .age,
            age
        );
        app.world_mut().resource_mut::<Session>().paused = false;
        app.world_mut().get_mut::<Player>(player).unwrap().pos.z = 12.;
        app.world_mut().get_mut::<Neighbor>(neighbor).unwrap().state = AlertState::Chase;
        app.world_mut().write_message(SoundCue::Alert);
        for _ in 0..60 {
            app.update();
        }
        assert_eq!(
            active(app.world().resource::<MusicDirector>()).theme,
            Theme::Chase
        );
        assert!(app.world().resource::<MusicDirector>().duck_gain < 0.5);
        let mut previous_theme = Theme::Chase;
        for _ in 0..3 {
            for outcome in [Outcome::Caught, Outcome::Escaped, Outcome::Exploring] {
                app.world_mut().resource_mut::<Session>().outcome = outcome;
                for _ in 0..240 {
                    app.update();
                    let count = app
                        .world_mut()
                        .query::<&MusicDeck>()
                        .iter(app.world())
                        .count();
                    assert!((1..=2).contains(&count));
                }
                let theme = active(app.world().resource::<MusicDirector>()).theme;
                assert_ne!(theme, previous_theme);
                previous_theme = theme;
            }
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyM);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        assert!(!app.world().resource::<MusicDirector>().enabled);
        app.world_mut().resource_mut::<Session>().attempts += 1;
        app.world_mut().get_mut::<Neighbor>(neighbor).unwrap().state = AlertState::Patrol;
        app.world_mut().get_mut::<Player>(player).unwrap().pos.z = 20.;
        app.update();
        assert_eq!(
            active(app.world().resource::<MusicDirector>()).theme,
            Theme::Evening
        );
        assert!(!app.world().resource::<MusicDirector>().enabled);
        let playing = app
            .world_mut()
            .query::<&MusicDeck>()
            .iter(app.world())
            .collect::<Vec<_>>();
        assert_eq!(playing.len(), 1);
        assert_eq!(playing[0].attempt, 2);
    }
    #[test]
    fn every_original_arrangement_is_distinct_bounded_stereo_pcm_with_quiet_edges() {
        let mut fingerprints = std::collections::HashSet::new();
        for theme in Theme::ALL {
            for arrangement in 0..ARRANGEMENTS {
                let wav = compose(Track { theme, arrangement });
                assert_eq!(&wav[..4], b"RIFF");
                assert_eq!(&wav[8..12], b"WAVE");
                assert_eq!(u16::from_le_bytes(wav[22..24].try_into().unwrap()), 2);
                assert_eq!(
                    u32::from_le_bytes(wav[24..28].try_into().unwrap()),
                    SAMPLE_RATE
                );
                assert_eq!(
                    wav.len(),
                    44 + (TRACK_SECONDS * SAMPLE_RATE as f32) as usize * 4
                );
                assert_eq!(&wav[44..48], &[0; 4]);
                assert_eq!(&wav[wav.len() - 4..], &[0; 4]);
                let samples = wav[44..].as_chunks::<2>().0;
                let peak = samples
                    .iter()
                    .map(|s| i16::from_le_bytes(*s).unsigned_abs())
                    .max()
                    .unwrap();
                assert!(peak > 1000 && peak < 28000, "{theme:?}: {peak}");
                assert!(
                    wav[44..]
                        .as_chunks::<4>()
                        .0
                        .iter()
                        .any(|s| s[..2] != s[2..])
                );
                let hash = wav.iter().fold(0_u64, |hash, b| {
                    hash.wrapping_mul(31).wrapping_add(u64::from(*b))
                });
                assert!(
                    fingerprints.insert(hash),
                    "duplicate arrangement: {theme:?}"
                );
                let source = AudioSource { bytes: wav.into() };
                let decoder = source.decoder();
                assert_eq!(decoder.channels().get(), 2);
                assert_eq!(decoder.sample_rate().get(), SAMPLE_RATE);
                assert_eq!(
                    decoder.total_duration().unwrap().as_secs(),
                    TRACK_SECONDS as u64
                );
            }
        }
    }
}
