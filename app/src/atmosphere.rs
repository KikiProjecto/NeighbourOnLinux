//! A continuous simulated day. Atmosphere is a style; night is only one time of day.
use crate::{
    LaunchOptions,
    game::{Outcome, Session},
};
use bevy::{ecs::system::SystemParam, prelude::*};

pub const DEFAULT_DAY_LENGTH: f32 = 120.;

pub struct AtmospherePlugin;
impl Plugin for AtmospherePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldClock>()
            .add_systems(Startup, initialize_clock)
            .add_systems(
                Update,
                (advance_clock, light_the_world)
                    .chain()
                    .before(crate::game::GameSet::Presentation),
            );
    }
}
#[derive(Resource, Debug)]
pub struct WorldClock {
    pub hour: f32,
    pub day: u32,
    pub day_length: f32,
}
impl Default for WorldClock {
    fn default() -> Self {
        Self {
            hour: 16.5,
            day: 1,
            day_length: DEFAULT_DAY_LENGTH,
        }
    }
}
impl WorldClock {
    pub fn advance(&mut self, seconds: f32) {
        let total = self.hour + seconds.max(0.) * 24. / self.day_length;
        self.day += (total / 24.).floor() as u32;
        self.hour = total.rem_euclid(24.);
    }
    pub fn phase(&self) -> &'static str {
        match self.hour {
            h if (5.0..11.0).contains(&h) => "MORNING",
            h if (11.0..16.0).contains(&h) => "AFTERNOON",
            h if (16.0..19.0).contains(&h) => "EVENING",
            _ => "NIGHT",
        }
    }
}
#[derive(Component)]
pub struct SkyLight;
#[derive(Component)]
pub enum CelestialBody {
    Moon,
    Star,
}
#[derive(Clone, Copy)]
pub struct SkySample {
    pub sky: Color,
    pub fog: Color,
    pub ambient: Color,
    pub brightness: f32,
    pub light: Color,
    pub lux: f32,
    pub direction: Vec3,
}
fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0., 1.);
    t * t * (3. - 2. * t)
}
fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> Color {
    Color::srgb(
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    )
}
pub fn sample_sky(hour: f32) -> SkySample {
    let sun_angle = (hour - 6.) / 24. * std::f32::consts::TAU;
    let altitude = sun_angle.sin();
    let daylight = smoothstep(-0.12, 0.3, altitude);
    let sunset = (1. - smoothstep(0.0, 0.5, altitude)) * daylight;
    let sky = mix([0.018, 0.027, 0.052], [0.27, 0.43, 0.56], daylight);
    let sky = sky.to_srgba();
    let direction = Vec3::new(sun_angle.cos() * 0.65, altitude.abs().max(0.15), 0.45).normalize();
    SkySample {
        sky: mix(
            [sky.red, sky.green, sky.blue],
            [0.38, 0.23, 0.19],
            sunset * 0.65,
        ),
        fog: mix([0.035, 0.05, 0.075], [0.40, 0.48, 0.52], daylight),
        ambient: mix([0.4, 0.53, 0.78], [0.69, 0.73, 0.72], daylight),
        brightness: 85. + daylight * 170.,
        light: mix(
            [0.5, 0.64, 0.9],
            [1., 0.89 - sunset * 0.28, 0.7 - sunset * 0.3],
            daylight,
        ),
        lux: 1400. + altitude.max(0.) * 14_000.,
        direction,
    }
}
pub(crate) fn initialize_clock(options: Res<LaunchOptions>, mut clock: ResMut<WorldClock>) {
    clock.hour = options.start_hour;
    clock.day_length = options.day_length;
}
fn advance_clock(time: Res<Time>, session: Res<Session>, mut clock: ResMut<WorldClock>) {
    if !session.paused && session.outcome == Outcome::Exploring {
        clock.advance(time.delta_secs());
    }
}
type KeyLightFilter = (With<SkyLight>, Without<CelestialBody>);
#[derive(SystemParam)]
struct LightingState<'w> {
    clock: Res<'w, WorldClock>,
    clear: ResMut<'w, ClearColor>,
    ambient: ResMut<'w, GlobalAmbientLight>,
}
fn light_the_world(
    state: LightingState,
    players: Query<&crate::player::Player>,
    mut lights: Query<(&mut DirectionalLight, &mut Transform), KeyLightFilter>,
    mut fogs: Query<&mut DistanceFog>,
    mut exposures: Query<&mut bevy::camera::Exposure, With<crate::game::GameCamera>>,
    mut bodies: Query<(&CelestialBody, &mut Visibility, &mut Transform), Without<SkyLight>>,
) {
    let LightingState {
        clock,
        mut clear,
        mut ambient,
    } = state;
    let underground = players
        .single()
        .map_or(0., |player| (-player.pos.y / 2.8).clamp(0., 1.));
    let sample = sample_sky(clock.hour);
    clear.0 = sample.sky;
    ambient.color = sample.ambient;
    ambient.brightness = sample.brightness + (100. - sample.brightness) * underground;
    for (mut light, mut transform) in &mut lights {
        light.color = sample.light;
        light.illuminance = sample.lux * (1. - underground * 0.98);
        *transform =
            Transform::from_translation(sample.direction * 60.).looking_at(Vec3::ZERO, Vec3::Y);
    }
    for mut exposure in &mut exposures {
        let ev100 = 9.7 - underground * 1.7;
        if exposure.ev100 != ev100 {
            exposure.ev100 = ev100;
        }
    }
    let night = !(5.5..18.5).contains(&clock.hour);
    for (body, mut visible, mut transform) in &mut bodies {
        *visible = if night {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        if matches!(body, CelestialBody::Moon) {
            let angle = (clock.hour - 18.) / 24. * std::f32::consts::TAU;
            transform.translation =
                Vec3::new(-angle.cos() * 58., angle.sin().abs() * 45. + 8., -55.);
        }
    }
    for mut fog in &mut fogs {
        fog.color = sample
            .fog
            .mix(&Color::srgb(0.025, 0.035, 0.039), underground);
        fog.falloff = FogFalloff::Linear {
            start: 45. + (13. - 45.) * underground,
            end: 210. + (29. - 210.) * underground,
        };
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn time_progresses_through_all_four_phases_and_rolls_into_next_day() {
        let mut clock = WorldClock {
            hour: 6.,
            day: 1,
            day_length: 240.,
        };
        assert_eq!(clock.phase(), "MORNING");
        clock.advance(60.);
        assert_eq!(clock.phase(), "AFTERNOON");
        clock.advance(50.);
        assert_eq!(clock.phase(), "EVENING");
        clock.advance(40.);
        assert_eq!(clock.phase(), "NIGHT");
        clock.advance(90.);
        assert_eq!(clock.day, 2);
        assert!((clock.hour - 6.).abs() < 0.01);
    }
    #[test]
    fn noon_is_brighter_than_night_and_sky_changes_continuously() {
        assert!(sample_sky(12.).lux > sample_sky(0.).lux * 5.);
        assert!((sample_sky(18.).brightness - sample_sky(18.001).brightness).abs() < 1.);
        let mut c = WorldClock::default();
        let before = c.hour;
        c.advance(0.);
        assert_eq!(before, c.hour);
    }
    #[test]
    fn default_day_completes_in_two_minutes_and_matches_launch_options() {
        let mut clock = WorldClock::default();
        assert_eq!(clock.day_length, 120.);
        assert_eq!(clock.day_length, LaunchOptions::default().day_length);
        let start = clock.hour;
        clock.advance(120.);
        assert_eq!(clock.day, 2);
        assert!((clock.hour - start).abs() < 0.001);
    }
}
