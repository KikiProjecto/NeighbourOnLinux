mod ai;
mod atmosphere;
mod audio;
mod basement;
mod capture;
mod character;
mod effects;
mod game;
mod house;
mod items;
mod materials;
mod motion;
mod music;
mod performance;
mod player;
mod props;
mod ui;
mod visuals;
mod world;

use bevy::{prelude::*, window::PresentMode};
use game::NeighborhoodPlugin;

#[derive(Resource, Debug)]
pub struct LaunchOptions {
    pub screenshot: Option<String>,
    pub frames: Option<u32>,
    pub overview: bool,
    pub quiet: bool,
    pub low_graphics: bool,
    pub music_volume: f32,
    pub ambience_volume: f32,
    pub effects_volume: f32,
    pub start_hour: f32,
    pub day_length: f32,
    pub inspect: Option<Inspection>,
    pub showcase: bool,
    pub profile_encounter: Option<String>,
    pub controls: bool,
}

impl Default for LaunchOptions {
    fn default() -> Self {
        Self {
            screenshot: None,
            frames: None,
            overview: false,
            quiet: false,
            low_graphics: false,
            music_volume: 0.7,
            ambience_volume: 1.,
            effects_volume: 1.,
            start_hour: 16.5,
            day_length: atmosphere::DEFAULT_DAY_LENGTH,
            inspect: None,
            showcase: false,
            profile_encounter: None,
            controls: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Inspection {
    Workshop,
    Archive,
    Station,
    Tower,
    Kitchen,
    LivingRoom,
    Bathroom,
    Gallery,
    Neighbor,
    Basement,
    BasementLab,
    BasementVault,
}
impl std::str::FromStr for Inspection {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "workshop" => Ok(Self::Workshop),
            "archive" => Ok(Self::Archive),
            "station" => Ok(Self::Station),
            "tower" => Ok(Self::Tower),
            "kitchen" => Ok(Self::Kitchen),
            "living-room" => Ok(Self::LivingRoom),
            "bathroom" => Ok(Self::Bathroom),
            "gallery" => Ok(Self::Gallery),
            "neighbor" => Ok(Self::Neighbor),
            "basement" => Ok(Self::Basement),
            "basement-lab" => Ok(Self::BasementLab),
            "basement-vault" => Ok(Self::BasementVault),
            _ => Err("--view expects workshop, archive, station, tower, kitchen, living-room, bathroom, gallery, neighbor, basement, basement-lab or basement-vault".into()),
        }
    }
}

fn main() -> AppExit {
    let (options, self_test) = match parse_args(std::env::args().skip(1)) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };
    if self_test {
        capture::self_test();
        return AppExit::Success;
    }
    let profiling = options.profile_encounter.is_some();
    let quiet = options.quiet;
    let low_graphics = options.low_graphics;
    let mut app = App::new();
    app.insert_resource(options)
        .insert_resource(ClearColor(Color::srgb(0.025, 0.039, 0.068)))
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.46, 0.59, 0.82),
            brightness: 115.0,
            ..default()
        })
        .insert_resource(Time::<Fixed>::from_hz(60.0))
        // A rendering stall must not create fifteen catch-up physics ticks and
        // another long input stall. Six fixed ticks is the per-frame ceiling.
        .insert_resource(Time::<Virtual>::from_max_delta(
            std::time::Duration::from_millis(100),
        ))
        .add_plugins({
            let plugins = DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Hello Neighbor · The impossible house | Bevy".into(),
                    resolution: if low_graphics {
                        (960, 540).into()
                    } else {
                        (1280, 720).into()
                    },
                    present_mode: if profiling {
                        PresentMode::AutoNoVsync
                    } else {
                        PresentMode::AutoVsync
                    },
                    ..default()
                }),
                ..default()
            });
            if quiet {
                plugins.disable::<bevy::audio::AudioPlugin>()
            } else {
                plugins
            }
        });
    if quiet {
        // Quiet capture/replay must not open an ALSA device or create an idle
        // audio callback. Keep the asset store for the gameplay plugin's data.
        app.init_asset::<AudioSource>();
    }
    app.add_plugins((
        NeighborhoodPlugin,
        basement::BasementPlugin,
        props::PropsPlugin,
        character::CharacterPlugin,
        effects::EffectsPlugin,
        atmosphere::AtmospherePlugin,
        visuals::VisualStylePlugin,
        ui::HudPlugin,
        audio::SoundPlugin,
        capture::CapturePlugin,
    ));
    if profiling {
        app.add_plugins(performance::EncounterProfilePlugin);
    }
    app.run()
}

fn parse_args(mut args: impl Iterator<Item = String>) -> Result<(LaunchOptions, bool), String> {
    let mut options = LaunchOptions::default();
    let mut self_test = false;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--self-test" => self_test = true,
            "--overview" => options.overview = true,
            "--quiet" => options.quiet = true,
            "--low-graphics" => options.low_graphics = true,
            "--controls" => options.controls = true,
            "--profile-encounter" => {
                options.profile_encounter = Some(
                    args.next()
                        .ok_or("--profile-encounter requires a CSV path")?,
                );
            }
            "--music-volume" | "--ambience-volume" | "--effects-volume" => {
                let value: f32 = args
                    .next()
                    .ok_or_else(|| format!("{flag} requires a number in [0,1]"))?
                    .parse()
                    .map_err(|_| format!("{flag} requires a number in [0,1]"))?;
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    return Err(format!("{flag} must be in [0,1]"));
                }
                match flag.as_str() {
                    "--music-volume" => options.music_volume = value,
                    "--ambience-volume" => options.ambience_volume = value,
                    _ => options.effects_volume = value,
                }
            }
            "--showcase" => options.showcase = true,
            "--view" => {
                options.inspect = Some(args.next().ok_or("--view requires a room name")?.parse()?);
                options.overview = true;
            }
            "--screenshot" => {
                options.screenshot = Some(args.next().ok_or("--screenshot requires a PNG path")?);
            }
            "--frames" => {
                let value = args.next().ok_or("--frames requires a positive integer")?;
                let frames: u32 = value
                    .parse()
                    .map_err(|_| "--frames requires a positive integer")?;
                if frames == 0 {
                    return Err("--frames must be greater than zero".into());
                }
                options.frames = Some(frames);
            }
            "--time" | "--day-length" => {
                let value: f32 = args
                    .next()
                    .ok_or_else(|| format!("{flag} requires a number"))?
                    .parse()
                    .map_err(|_| format!("{flag} requires a number"))?;
                if !value.is_finite() {
                    return Err(format!("{flag} must be finite"));
                }
                if flag == "--time" {
                    if !(0.0..24.0).contains(&value) {
                        return Err("--time must be in [0, 24)".into());
                    }
                    options.start_hour = value;
                } else {
                    if value < 24. {
                        return Err("--day-length must be at least 24 seconds".into());
                    }
                    options.day_length = value;
                }
            }
            "--help" | "-h" => {
                println!(
                    "hello-neighbor [--self-test] [--screenshot file.png] [--overview] [--frames N] [--quiet] [--low-graphics] [--controls] [--music-volume 0..1] [--ambience-volume 0..1] [--effects-volume 0..1] [--view workshop|archive|station|tower|kitchen|living-room|bathroom|gallery|neighbor|basement|basement-lab|basement-vault] [--showcase] [--time HOUR] [--day-length SECONDS] [--profile-encounter report.csv]\nWASD move · mouse look · Space jump · Shift run · Ctrl crouch · E interact\nQ drop · G throw · Tab basement map · F flashlight · H help · M music on/off · Esc release/pause · click resume · R retry"
                );
                std::process::exit(0);
            }
            _ => return Err(format!("Unknown argument: {flag}. Use --help.")),
        }
    }
    if options.inspect.is_some() && options.screenshot.is_none() {
        return Err("--view is a screenshot inspection option; add --screenshot file.png".into());
    }
    if options.showcase && (options.screenshot.is_none() || options.inspect.is_none()) {
        return Err("--showcase requires --view and --screenshot".into());
    }
    if options.profile_encounter.is_some()
        && (options.screenshot.is_some()
            || options.overview
            || options.frames.is_some()
            || self_test)
    {
        return Err("--profile-encounter runs its own timed play scenario; omit --screenshot, --overview, --view, --frames and --self-test".into());
    }
    Ok((options, self_test))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_incomplete_or_invalid_cli_options() {
        for args in [
            vec!["--screenshot"],
            vec!["--showcase"],
            vec!["--frames", "0"],
            vec!["--frames", "oops"],
            vec!["--typo"],
            vec!["--view", "unknown"],
            vec!["--view", "tower"],
            vec!["--time", "24"],
            vec!["--time", "-1"],
            vec!["--time", "NaN"],
            vec!["--day-length", "0"],
            vec!["--day-length", "inf"],
            vec!["--music-volume"],
            vec!["--music-volume", "NaN"],
            vec!["--music-volume", "-0.1"],
            vec!["--ambience-volume", "inf"],
            vec!["--effects-volume", "1.1"],
            vec!["--effects-volume", "oops"],
        ] {
            assert!(parse_args(args.into_iter().map(str::to_owned)).is_err());
        }
    }
    #[test]
    fn audio_categories_have_independent_validated_volume_controls() {
        let (options, _) = parse_args(
            [
                "--music-volume",
                "0",
                "--ambience-volume",
                "0.4",
                "--effects-volume",
                "1",
            ]
            .into_iter()
            .map(str::to_owned),
        )
        .unwrap();
        assert_eq!(options.music_volume, 0.);
        assert_eq!(options.ambience_volume, 0.4);
        assert_eq!(options.effects_volume, 1.);
        assert!(LaunchOptions::default().music_volume > 0.);
    }
    #[test]
    fn low_graphics_mode_is_opt_in() {
        let defaults = LaunchOptions::default();
        assert!(!defaults.low_graphics);
        let (options, _) = parse_args(["--low-graphics"].into_iter().map(str::to_owned)).unwrap();
        assert!(options.low_graphics);
    }
}
