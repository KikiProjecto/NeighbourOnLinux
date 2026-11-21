//! Tiny original bitmap lettering keeps underground signs readable without external assets.
use super::{ROOMS, UndergroundMesh};
use bevy::{
    asset::RenderAssetUsages,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat},
};
const LABELS: [&str; 12] = [
    "01 / BOILER ROOM",
    "02 / MAINTENANCE",
    "03 / PUMP STATION",
    "04 / GENERATOR VAULT",
    "05 / CISTERN",
    "06 / COLD STORAGE",
    "07 / RECORDS ARCHIVE",
    "08 / OBSERVATION",
    "09 / VENTILATION",
    "10 / LABORATORY",
    "11 / EVIDENCE VAULT",
    "12 / SERVICE EXIT",
];
fn glyph(letter: char) -> [u8; 7] {
    match letter {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [15, 16, 16, 16, 16, 16, 15],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [15, 16, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [31, 4, 4, 4, 4, 4, 31],
        'J' => [7, 2, 2, 2, 18, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 25, 21, 19, 17, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [30, 1, 1, 14, 1, 1, 30],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 16, 30, 1, 1, 30],
        '6' => [14, 16, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 1, 14],
        '/' => [1, 2, 2, 4, 8, 8, 16],
        _ => [0; 7],
    }
}
fn texture(text: &str) -> Image {
    const W: usize = 384;
    const H: usize = 48;
    const SCALE: usize = 2;
    let mut bytes = vec![0_u8; W * H * 4];
    for y in 0..H {
        for x in 0..W {
            let border = !(3..W - 3).contains(&x) || !(3..H - 3).contains(&y);
            let pixel = if border {
                [166, 160, 125, 255]
            } else {
                [29, 54, 57, 255]
            };
            bytes[(y * W + x) * 4..(y * W + x + 1) * 4].copy_from_slice(&pixel);
        }
    }
    let start = (W - text.len() * 6 * SCALE) / 2;
    for (letter_index, letter) in text.chars().enumerate() {
        for (y, row) in glyph(letter).into_iter().enumerate() {
            for x in 0..5 {
                if row & (1 << (4 - x)) != 0 {
                    for dy in 0..SCALE {
                        for dx in 0..SCALE {
                            let index = ((17 + y * SCALE + dy) * W
                                + start
                                + letter_index * 6 * SCALE
                                + x * SCALE
                                + dx)
                                * 4;
                            bytes[index..index + 4].copy_from_slice(&[237, 223, 174, 255]);
                        }
                    }
                }
            }
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: W as u32,
            height: H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        bytes,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = bevy::image::ImageSampler::nearest();
    image
}
pub(super) fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let mesh = meshes.add(Rectangle::new(3.4, 0.425));
    for (room, label) in ROOMS.into_iter().zip(LABELS) {
        let center = room.center() + Vec3::new(0., 2.64, -4.1);
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(images.add(texture(label))),
            unlit: true,
            ..default()
        });
        commands.spawn((
            Name::new("English underground room sign"),
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(center),
            UndergroundMesh {
                center,
                radius: 1.8,
            },
        ));
    }
}
