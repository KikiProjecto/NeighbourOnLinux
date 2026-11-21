//! A copied maintenance map, opened by reading the entrance board and pressing Tab.
use super::{BasementProgress, CELL, FLOOR, Labyrinth, ROOMS, WIDTH, cell_center};
use crate::{
    game::{GameCamera, GameSet, Outcome, Session},
    player::Player,
};
use bevy::prelude::*;
const STEP: f32 = 29.;
const INK: Color = Color::srgb(0.9, 0.87, 0.76);
#[derive(Component)]
struct MapPanel;
#[derive(Component)]
struct MapPlayer;
#[derive(Component)]
struct MapBreaker(usize);
#[derive(Component)]
struct MapStatus;
pub(super) fn install(app: &mut App) {
    app.add_systems(Startup, setup.after(GameSet::Setup))
        .add_systems(
            Update,
            update.after(super::controls).after(GameSet::Presentation),
        );
}
fn label(value: &str, size: f32) -> impl Bundle {
    (
        Text::new(value),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(INK),
    )
}
fn rectangle(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: px(left),
        top: px(top),
        width: px(width),
        height: px(height),
        ..default()
    }
}
fn setup(mut commands: Commands, cameras: Query<Entity, With<GameCamera>>) {
    let Ok(camera) = cameras.single() else {
        return;
    };
    let maze = Labyrinth::default();
    commands
        .spawn((
            Name::new("Underground maintenance map"),
            UiTargetCamera(camera),
            MapPanel,
            Visibility::Hidden,
            GlobalZIndex(10),
            Node {
                position_type: PositionType::Absolute,
                left: px(24),
                top: px(87),
                width: px(STEP * WIDTH as f32 + 32.),
                padding: UiRect::all(px(16)),
                row_gap: px(9),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(Color::srgba(0.028, 0.048, 0.051, 0.97)),
        ))
        .with_children(|panel| {
            panel.spawn(label("MAINTENANCE / UNDERGROUND", 14.));
            panel.spawn(label("NORTH ^     TAB - close map", 11.));
            panel
                .spawn((
                    Node {
                        width: px(STEP * WIDTH as f32),
                        height: px(STEP * WIDTH as f32),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.07, 0.1, 0.1)),
                ))
                .with_children(|canvas| {
                    for z in 0..WIDTH {
                        for x in 0..WIDTH {
                            let left = x as f32 * STEP;
                            let top = z as f32 * STEP;
                            let wall = Color::srgb(0.44, 0.49, 0.44);
                            if x == 0 {
                                canvas
                                    .spawn((rectangle(left, top, 2., STEP), BackgroundColor(wall)));
                            }
                            if z == 0 {
                                canvas
                                    .spawn((rectangle(left, top, STEP, 2.), BackgroundColor(wall)));
                            }
                            if maze.passages[z][x] & 2 == 0 {
                                canvas.spawn((
                                    rectangle(left + STEP - 2., top, 2., STEP),
                                    BackgroundColor(wall),
                                ));
                            }
                            if maze.passages[z][x] & 4 == 0 {
                                canvas.spawn((
                                    rectangle(left, top + STEP - 2., STEP, 2.),
                                    BackgroundColor(wall),
                                ));
                            }
                        }
                    }
                    for (index, room) in super::CIRCUIT_ROOMS.into_iter().enumerate() {
                        let room = ROOMS[room];
                        canvas.spawn((
                            MapBreaker(index),
                            rectangle(
                                (room.x + 1) as f32 * STEP - 5.,
                                (room.z + 1) as f32 * STEP - 5.,
                                10.,
                                10.,
                            ),
                            BackgroundColor(
                                [
                                    Color::srgb(0.95, 0.46, 0.16),
                                    Color::srgb(0.16, 0.7, 0.75),
                                    Color::srgb(0.7, 0.4, 0.84),
                                ][index],
                            ),
                        ));
                    }
                    for (index, symbol) in [(10, "K"), (11, "X")] {
                        let room = ROOMS[index];
                        canvas.spawn((
                            label(symbol, 14.),
                            rectangle(
                                (room.x + 1) as f32 * STEP - 5.,
                                (room.z + 1) as f32 * STEP - 9.,
                                14.,
                                18.,
                            ),
                        ));
                    }
                    canvas.spawn((
                        MapPlayer,
                        rectangle(0., 0., 7., 7.),
                        BackgroundColor(Color::srgb(1., 0.84, 0.3)),
                    ));
                });
            panel.spawn(label("ORANGE boiler   TEAL generator   PURPLE air", 10.));
            panel.spawn(label("YELLOW you     K service key     X exit", 11.));
            panel.spawn((MapStatus, label("", 11.)));
        });
}
fn update(
    progress: Res<BasementProgress>,
    session: Res<Session>,
    players: Query<&Player>,
    mut panels: Query<&mut Visibility, With<MapPanel>>,
    mut markers: Query<&mut Node, With<MapPlayer>>,
    mut breakers: Query<(&MapBreaker, &mut BackgroundColor)>,
    mut status: Query<&mut Text, With<MapStatus>>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let visible = !session.paused
        && progress.map_open
        && player.pos.y < FLOOR + 3.
        && session.outcome == Outcome::Exploring;
    for mut panel in &mut panels {
        panel.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if !visible {
        return;
    }
    let origin = cell_center(0, 0) - Vec3::splat(CELL * 0.5);
    for mut marker in &mut markers {
        let left =
            px(((player.pos.x - origin.x) / CELL * STEP - 3.5).clamp(0., STEP * WIDTH as f32 - 7.));
        let top =
            px(((player.pos.z - origin.z) / CELL * STEP - 3.5).clamp(0., STEP * WIDTH as f32 - 7.));
        if marker.left != left {
            marker.left = left;
        }
        if marker.top != top {
            marker.top = top;
        }
    }
    for (breaker, mut color) in &mut breakers {
        color.set_if_neq(BackgroundColor(if progress.circuits[breaker.0] {
            Color::srgb(0.62, 0.9, 0.54)
        } else {
            [
                Color::srgb(0.95, 0.46, 0.16),
                Color::srgb(0.16, 0.7, 0.75),
                Color::srgb(0.7, 0.4, 0.84),
            ][breaker.0]
        }));
    }
    let value = format!(
        "CIRCUITS {}/3  /  SERVICE KEY {}",
        progress.circuits.iter().filter(|on| **on).count(),
        if progress.service_key {
            "collected"
        } else {
            "in evidence vault"
        }
    );
    for mut text in &mut status {
        if text.0 != value {
            text.0 = value.clone();
        }
    }
}
