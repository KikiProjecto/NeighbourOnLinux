//! Compact English HUD: location, objective, awareness and inventory.
use crate::{
    LaunchOptions,
    ai::Neighbor,
    game::{GameCamera, GameSet, Outcome, Session},
    items::{Inventory, PuzzleProgress},
    player::Player,
    world::HousePlan,
};
use bevy::{ecs::system::SystemParam, prelude::*};
pub struct HudPlugin;
impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HudMode>()
            .add_systems(Startup, setup_hud.after(GameSet::Setup))
            .add_systems(
                Update,
                (toggle_help, update_hud)
                    .chain()
                    .after(GameSet::Presentation),
            );
    }
}
const INK: Color = Color::srgb(0.86, 0.85, 0.79);
const MUTED: Color = Color::srgb(0.58, 0.64, 0.65);
const ACCENT: Color = Color::srgb(0.82, 0.47, 0.26);
#[derive(Component)]
enum HudText {
    Location,
    Objective,
    Notice,
    Prompt,
    Inventory,
    Neighbor,
    Slot(u8),
    Controls,
    OverlayTitle,
    OverlayBody,
}
#[derive(Resource, Default)]
struct HudMode {
    expanded: bool,
}
#[derive(Component)]
struct FocusReticle;
fn toggle_help(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<HudMode>) {
    if keys.just_pressed(KeyCode::KeyH) {
        mode.expanded = !mode.expanded;
    }
}
#[derive(Component)]
struct AwarenessBar;
#[derive(Component)]
struct Overlay;
#[derive(Component)]
struct ControlsGuide;
fn text(value: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(font_safe(&value.into())),
        TextFont::from_font_size(size),
        TextColor(color),
    )
}
// The embedded default font has a deliberately small glyph repertoire.
fn font_safe(value: &str) -> String {
    value
        .replace('·', "/")
        .replace('—', "-")
        .replace('…', "...")
}
fn setup_hud(
    mut commands: Commands,
    cameras: Query<Entity, With<GameCamera>>,
    options: Res<LaunchOptions>,
    mut mode: ResMut<HudMode>,
) {
    mode.expanded = options.controls;
    let Ok(camera) = cameras.single() else {
        return;
    };
    commands
        .spawn((
            Name::new("HUD"),
            UiTargetCamera(camera),
            Node {
                width: percent(100),
                height: percent(100),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: px(24),
                left: px(28),
                flex_direction: FlexDirection::Column,
                row_gap: px(6),
                max_width: percent(65),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn(text("FRIENDLY COURT", 14., INK));
                panel.spawn((text("", 12., INK), HudText::Location));
            });
            root.spawn(Node {
                position_type: PositionType::Absolute,
                top: px(26),
                right: px(28),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::End,
                row_gap: px(8),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn((text("", 11., MUTED), HudText::Neighbor));
                panel
                    .spawn((
                        Node {
                            width: px(128),
                            height: px(3),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.25, 0.28, 0.29, 0.35)),
                    ))
                    .with_children(|bar| {
                        bar.spawn((
                            AwarenessBar,
                            Node {
                                width: percent(0),
                                height: percent(100),
                                ..default()
                            },
                            BackgroundColor(ACCENT),
                        ));
                    });
            });
            root.spawn(Node {
                width: percent(100),
                height: percent(100),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            })
            .with_children(|center| {
                center.spawn((
                    FocusReticle,
                    Node {
                        width: px(3),
                        height: px(3),
                        ..default()
                    },
                    BackgroundColor(INK),
                ));
            });
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: px(135),
                width: percent(100),
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                padding: UiRect::horizontal(px(28)),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn((text("", 16., INK), HudText::Prompt));
                panel.spawn((text("", 13., Color::srgb(0.9, 0.78, 0.6)), HudText::Notice));
            });
            root.spawn(Node {
                position_type: PositionType::Absolute,
                bottom: px(24),
                left: px(28),
                max_width: percent(75),
                flex_direction: FlexDirection::Column,
                row_gap: px(8),
                ..default()
            })
            .with_children(|panel| {
                panel.spawn((text("", 13., INK), HudText::Objective));
                panel.spawn((text("", 11., MUTED), HudText::Inventory));
                panel
                    .spawn(Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(6),
                        ..default()
                    })
                    .with_children(|slots| {
                        for i in 0..4 {
                            slots
                                .spawn((
                                    Node {
                                        width: px(62),
                                        height: px(44),
                                        border: UiRect::all(px(1)),
                                        justify_content: JustifyContent::Center,
                                        align_items: AlignItems::Center,
                                        ..default()
                                    },
                                    BorderColor::all(Color::srgba(0.74, 0.73, 0.63, 0.35)),
                                    BackgroundColor(Color::srgba(0.02, 0.03, 0.035, 0.65)),
                                ))
                                .with_children(|slot| {
                                    slot.spawn((text("", 10., INK), HudText::Slot(i)));
                                });
                        }
                    });
                panel.spawn((text("", 11., MUTED), HudText::Controls));
            });
            root.spawn((
                ControlsGuide,
                GlobalZIndex(20),
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    right: px(28), top: px(100), width: px(350),
                    padding: UiRect::all(px(22)), row_gap: px(12),
                    flex_direction: FlexDirection::Column,
                    ..default()
                },
                BackgroundColor(Color::srgba(0.025, 0.04, 0.052, 0.94)),
            )).with_children(|guide| {
                guide.spawn(text("HOW TO PLAY", 17., ACCENT));
                guide.spawn(text("Pick up a box with E. Aim, then press G to throw it and distract the neighbor.", 14., INK));
                for row in [
                    "W A S D   Move / Mouse   Look",
                    "SHIFT   Hold to run / SPACE   Jump",
                    "CTRL or C   Crouch quietly",
                    "E   Pick up / Interact / Unlock",
                    "G   Throw held box / Q   Drop box",
                    "F   Flashlight / M   Music on or off",
                    "TAB   Underground map after reading board",
                    "SHIFT + umbrella   Slow your fall",
                    "ESC   Pause / Left click   Resume",
                    "R   Retry after capture or completion",
                    "F10   Quit / H   Close this guide",
                ] { guide.spawn(text(row, 13., INK)); }
            });
            root.spawn((
                Overlay,
                Visibility::Hidden,
                Node {
                    position_type: PositionType::Absolute,
                    width: percent(100),
                    height: percent(100),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    row_gap: px(20),
                    padding: UiRect::all(px(48)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.012, 0.02, 0.03, 0.94)),
            ))
            .with_children(|panel| {
                panel.spawn(text("THE IMPOSSIBLE HOUSE", 13., ACCENT));
                panel.spawn((text("", 42., INK), HudText::OverlayTitle));
                panel.spawn((text("", 17., MUTED), HudText::OverlayBody));
            });
        });
}
#[derive(SystemParam)]
struct HudState<'w> {
    session: Res<'w, Session>,
    mode: Res<'w, HudMode>,
    progress: Res<'w, PuzzleProgress>,
    inventory: Res<'w, Inventory>,
    plan: Res<'w, HousePlan>,
    basement: Res<'w, crate::basement::BasementProgress>,
}
#[derive(SystemParam)]
struct HudPanels<'w, 's> {
    overlays: Query<'w, 's, &'static mut Visibility, With<Overlay>>,
    guides: Query<'w, 's, &'static mut Visibility, (With<ControlsGuide>, Without<Overlay>)>,
}
fn update_hud(
    state: HudState,
    players: Query<&Player>,
    neighbors: Query<&Neighbor>,
    mut texts: Query<(&HudText, &mut Text)>,
    mut bars: Query<&mut Node, (With<AwarenessBar>, Without<FocusReticle>)>,
    mut panels: HudPanels,
    mut reticles: Query<(&mut Node, &mut BackgroundColor), With<FocusReticle>>,
) {
    let HudState {
        session,
        mode,
        progress,
        inventory,
        plan,
        basement,
    } = state;
    let Ok(p) = players.single() else {
        return;
    };
    let n = neighbors.single().ok();
    for (tag, mut text) in &mut texts {
        let value = match tag {
            HudText::Location => plan.room_at(p.pos + Vec3::Y).into(),
            HudText::Objective => {
                if mode.expanded {
                    if progress.basement_open {
                        basement.objective()
                    } else {
                        format!("HOUSE / {}", progress.objective(&inventory))
                    }
                } else {
                    String::new()
                }
            }
            HudText::Prompt => {
                if inventory.carrying.is_some() {
                    format!("{}    [G] Throw / [Q] Drop", session.prompt)
                } else {
                    session.prompt.clone()
                }
            }
            HudText::Notice => {
                if session.notice_for > 0. {
                    session.notice.clone()
                } else {
                    String::new()
                }
            }
            HudText::Inventory => format!(
                "KEYS [{}{}{}]{}   FLASHLIGHT {}",
                if inventory.blue_key { "B" } else { "-" },
                if inventory.green_key { "G" } else { "-" },
                if inventory.red_key { "R" } else { "-" },
                if basement.service_key {
                    " + SERVICE"
                } else {
                    ""
                },
                if session.flashlight { "ON" } else { "OFF" }
            ),
            HudText::Slot(slot) => match slot {
                0 if inventory.crowbar => "CROWBAR",
                1 if inventory.keycard => "KEYCARD",
                2 if inventory.umbrella => "UMBRELLA",
                3 if inventory.carrying.is_some() => "BOX",
                _ => "",
            }
            .into(),
            HudText::Controls => if mode.expanded {
                "H close controls"
            } else {
                "H controls / SHIFT run / E interact / G throw"
            }
            .into(),
            HudText::Neighbor => format!("PETERSON  /  {}", n.map_or("…", |n| n.state.label())),
            HudText::OverlayTitle => match session.outcome {
                Outcome::Caught => "HE FOUND YOU.",
                Outcome::Escaped => "YOU MADE IT OUT.",
                _ => "PAUSED",
            }
            .into(),
            HudText::OverlayBody => match session.outcome {
                Outcome::Caught => "R — try again. He will watch the place where you were caught.",
                Outcome::Escaped => {
                    "The circuits are restored. You escaped with the service key. R — play again."
                }
                _ => "Click to resume. F10 — quit.",
            }
            .into(),
        };
        let value = font_safe(&value);
        if text.0 != value {
            text.0 = value;
        }
    }
    for mut bar in &mut bars {
        let width = percent(n.map_or(0., |n| n.awareness * 100.));
        if bar.width != width {
            bar.width = width;
        }
    }
    for (mut node, mut color) in &mut reticles {
        let focused = !session.prompt.is_empty();
        let size = px(if focused { 7. } else { 3. });
        if node.width != size {
            node.width = size;
        }
        if node.height != size {
            node.height = size;
        }
        color.set_if_neq(BackgroundColor(if focused { ACCENT } else { INK }));
    }
    for mut overlay in &mut panels.overlays {
        overlay.set_if_neq(if session.paused || session.outcome != Outcome::Exploring {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for mut guide in &mut panels.guides {
        guide.set_if_neq(if mode.expanded {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}
