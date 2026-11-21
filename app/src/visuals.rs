//! Shared filmic grading and a small painted sky; no volumetric/cloud simulation.
use crate::{
    atmosphere::WorldClock,
    game::{GameCamera, GameSet},
};
use bevy::{
    asset::RenderAssetUsages,
    light::Skybox,
    prelude::*,
    render::{
        render_resource::{
            Extent3d, TextureDimension, TextureFormat, TextureViewDescriptor, TextureViewDimension,
        },
        view::{ColorGrading, ColorGradingGlobal, ColorGradingSection},
    },
};

pub struct VisualStylePlugin;
impl Plugin for VisualStylePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Startup,
            setup
                .after(GameSet::Setup)
                .after(crate::atmosphere::initialize_clock),
        )
        .add_systems(Update, paint_sky.after(GameSet::Presentation));
    }
}
pub(crate) fn color_grade() -> ColorGrading {
    ColorGrading {
        global: ColorGradingGlobal {
            exposure: 0.12,
            post_saturation: 1.14,
            ..default()
        },
        shadows: ColorGradingSection {
            contrast: 0.98,
            lift: 0.006,
            ..default()
        },
        midtones: ColorGradingSection {
            contrast: 1.05,
            saturation: 1.04,
            ..default()
        },
        highlights: ColorGradingSection {
            contrast: 1.02,
            ..default()
        },
    }
}
const SKY_SIZE: u32 = 64;
const SKY_STEPS: usize = 96;
#[derive(Resource)]
struct PaintedSky {
    images: Vec<Handle<Image>>,
    index: usize,
}
fn sky_texels() -> Vec<(f32, f32)> {
    let mut texels = Vec::with_capacity((SKY_SIZE * SKY_SIZE * 6) as usize);
    for face in 0..6 {
        for y in 0..SKY_SIZE {
            for x in 0..SKY_SIZE {
                let u = (x as f32 + 0.5) / SKY_SIZE as f32 * 2. - 1.;
                let v = (y as f32 + 0.5) / SKY_SIZE as f32 * 2. - 1.;
                let direction = match face {
                    0 => Vec3::new(1., -v, -u),
                    1 => Vec3::new(-1., -v, u),
                    2 => Vec3::new(u, 1., v),
                    3 => Vec3::new(u, -1., -v),
                    4 => Vec3::new(u, -v, 1.),
                    _ => Vec3::new(-u, -v, -1.),
                }
                .normalize();
                // Small overlapping painted puffs, distributed in horizontal sky bands.
                // Angular shapes remain continuous across cube-face boundaries.
                let azimuth = direction.z.atan2(direction.x);
                let mut cloud = 0_f32;
                for cluster in 0..14 {
                    let heading = cluster as f32 * 2.399_963;
                    let height = 0.16 + ((cluster * 7) % 11) as f32 * 0.035;
                    for puff in 0..3 {
                        let dx = (azimuth - heading - (puff as f32 - 1.) * 0.095
                            + std::f32::consts::PI)
                            .rem_euclid(std::f32::consts::TAU)
                            - std::f32::consts::PI;
                        let dy = direction.y - height - if puff == 1 { 0.027 } else { 0. };
                        let width = 0.1 + (cluster % 3) as f32 * 0.028;
                        let density = (-(dx / width).powi(2) - (dy / 0.052).powi(2)).exp();
                        cloud = cloud.max(density);
                    }
                }
                texels.push((direction.y, cloud));
            }
        }
    }
    texels
}
fn sky_bytes(texels: &[(f32, f32)], hour: f32) -> Vec<u8> {
    let altitude = ((hour - 6.) / 24. * std::f32::consts::TAU).sin();
    let day = ((altitude + 0.12) / 0.42).clamp(0., 1.);
    let sunset = (1. - (altitude.abs() / 0.45).clamp(0., 1.)) * day;
    let zenith = Vec3::new(0.012, 0.025, 0.065).lerp(Vec3::new(0.16, 0.36, 0.58), day);
    let horizon = Vec3::new(0.055, 0.075, 0.12)
        .lerp(Vec3::new(0.68, 0.77, 0.78), day)
        .lerp(Vec3::new(0.88, 0.48, 0.28), sunset * 0.7);
    let clouds = Vec3::new(0.06, 0.075, 0.11)
        .lerp(Vec3::new(0.84, 0.87, 0.83), day)
        .lerp(Vec3::new(0.91, 0.66, 0.45), sunset * 0.6);
    texels
        .iter()
        .flat_map(|&(height, cloud)| {
            let color = horizon
                .lerp(zenith, height.max(0.).sqrt())
                .lerp(clouds, cloud * 0.76);
            [
                (color.x * 255.) as u8,
                (color.y * 255.) as u8,
                (color.z * 255.) as u8,
                255,
            ]
        })
        .collect()
}
fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    clock: Res<WorldClock>,
    cameras: Query<Entity, With<GameCamera>>,
) {
    let texels = sky_texels();
    // Preload the small palette sequence at startup. Gameplay only switches
    // handles; it never allocates/reuploads a sky texture during an encounter.
    let frames: Vec<_> = (0..SKY_STEPS)
        .map(|step| {
            let mut image = Image::new(
                Extent3d {
                    width: SKY_SIZE,
                    height: SKY_SIZE,
                    depth_or_array_layers: 6,
                },
                TextureDimension::D2,
                sky_bytes(&texels, step as f32 * 24. / SKY_STEPS as f32),
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::RENDER_WORLD,
            );
            image.texture_view_descriptor = Some(TextureViewDescriptor {
                dimension: Some(TextureViewDimension::Cube),
                ..default()
            });
            images.add(image)
        })
        .collect();
    let index = (clock.hour / 24. * SKY_STEPS as f32) as usize % SKY_STEPS;
    for camera in &cameras {
        commands.entity(camera).insert(Skybox {
            image: Some(frames[index].clone()),
            brightness: 1200.,
            ..default()
        });
    }
    commands.insert_resource(PaintedSky {
        images: frames,
        index,
    });
}
fn paint_sky(
    clock: Res<WorldClock>,
    mut sky: ResMut<PaintedSky>,
    mut cameras: Query<&mut Skybox, With<GameCamera>>,
) {
    let index = (clock.hour / 24. * SKY_STEPS as f32) as usize % SKY_STEPS;
    if sky.index == index {
        return;
    }
    sky.index = index;
    for mut camera in &mut cameras {
        camera.image = Some(sky.images[index].clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn painted_sky_has_six_faces_and_distinct_day_dusk_night_colors() {
        let texels = sky_texels();
        let day = sky_bytes(&texels, 12.);
        let dusk = sky_bytes(&texels, 18.);
        let night = sky_bytes(&texels, 0.);
        assert_eq!(day.len(), (SKY_SIZE * SKY_SIZE * 6 * 4) as usize);
        assert_ne!(day, dusk);
        assert_ne!(dusk, night);
        assert!(
            day.iter().map(|v| u64::from(*v)).sum::<u64>()
                > night.iter().map(|v| u64::from(*v)).sum::<u64>()
        );
        assert!(day.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 255));
    }
}
