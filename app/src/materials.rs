//! Small shared procedural surface maps; no per-object texture allocation.
use crate::world::Paint;
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};

pub fn surface(paint: Paint, images: &mut Assets<Image>) -> StandardMaterial {
    let textured = matches!(
        paint,
        Paint::Wood
            | Paint::DarkWood
            | Paint::Trunk
            | Paint::Roof
            | Paint::Blue
            | Paint::PaleBlue
            | Paint::Teal
            | Paint::Cream
            | Paint::Wallpaper
            | Paint::Concrete
            | Paint::Grass
            | Paint::Vest
            | Paint::Pants
    );
    let texture = textured.then(|| {
        let mut bytes = Vec::with_capacity(32 * 32 * 4);
        for y in 0..32_u32 {
            for x in 0..32_u32 {
                let hash = x.wrapping_mul(73_856_093) ^ y.wrapping_mul(19_349_663);
                let grain = match paint {
                    Paint::Wood | Paint::DarkWood | Paint::Trunk => {
                        ((y as f32 * 1.2 + (x as f32 * 0.22).sin()).sin() * 9.) as i16
                    }
                    Paint::Vest | Paint::Pants => {
                        if (x + y) % 2 == 0 {
                            4
                        } else {
                            -5
                        }
                    }
                    Paint::Wallpaper => {
                        if x % 8 == 0 || y % 16 == 0 {
                            -12
                        } else {
                            2
                        }
                    }
                    Paint::Roof if y % 8 == 0 || (x + (y / 8) * 8) % 16 == 0 => -16,
                    _ => 0,
                };
                let value = (242 + (hash % 11) as i16 - 5 + grain).clamp(0, 255) as u8;
                bytes.extend_from_slice(&[value, value, value, 255]);
            }
        }
        images.add(Image::new(
            Extent3d {
                width: 32,
                height: 32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            bytes,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        ))
    });
    StandardMaterial {
        base_color: paint.color(),
        base_color_texture: texture,
        perceptual_roughness: match paint {
            Paint::Metal => 0.38,
            Paint::Glass => 0.19,
            Paint::Skin => 0.7,
            _ => 0.88,
        },
        metallic: if matches!(paint, Paint::Metal) {
            0.65
        } else {
            0.
        },
        emissive: if matches!(paint, Paint::Glow) {
            LinearRgba::rgb(3., 1.5, 0.4)
        } else {
            LinearRgba::BLACK
        },
        ..default()
    }
}
