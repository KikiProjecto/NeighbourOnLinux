//! Reusable household interactions. Authored pieces become stateful render hierarchies;
//! decorative animation never changes the player's collision controller.
use crate::{
    ai::Neighbor,
    audio::SoundCue,
    game::{Art, GameSet, Outcome, Session, can_interact},
    player::Player,
    world::{CollisionWorld, FLOOR_Y, Paint, Piece, STOREY, Shape},
};
use bevy::prelude::*;

pub struct PropsPlugin;
impl Plugin for PropsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, stage_showcase.after(GameSet::Setup))
            .add_systems(
                Update,
                (interact_props, animate_props)
                    .chain()
                    .after(GameSet::Interact)
                    .before(GameSet::Presentation),
            );
    }
}

fn stage_showcase(
    options: Option<Res<crate::LaunchOptions>>,
    mut props: Query<&mut InteractiveProp>,
) {
    if options.is_some_and(|o| o.showcase) {
        for mut prop in &mut props {
            prop.active = true;
            prop.dial = 1;
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PropKind {
    Television,
    Telephone,
    Lamp,
    Drawer,
    Wardrobe,
    Book,
    Apple,
    Painting,
    Sink,
    Shower,
    Kettle,
    Mailbox,
    Doorbell,
    Dial,
}
impl PropKind {
    pub fn from_piece(name: &str) -> Option<Self> {
        Some(match name {
            "CRT screen" => Self::Television,
            "Telephone" => Self::Telephone,
            "Lamp shade" => Self::Lamp,
            "Drawer pull" | "Dresser drawer" => Self::Drawer,
            "Wardrobe doors" => Self::Wardrobe,
            "Book" => Self::Book,
            "Apple" => Self::Apple,
            "Crooked painting frame" => Self::Painting,
            "Sink" => Self::Sink,
            "Shower valve" => Self::Shower,
            "Red kettle" => Self::Kettle,
            "Mailbox flag" => Self::Mailbox,
            "Porch doorbell" => Self::Doorbell,
            "Signal dial" => Self::Dial,
            _ => return None,
        })
    }
    fn label(self, on: bool) -> &'static str {
        match (self, on) {
            (Self::Television, false) => "Turn on TV / create a distraction",
            (Self::Television, true) => "Turn off TV",
            (Self::Telephone, _) => "Ring telephone / create a distraction",
            (Self::Lamp, false) => "Turn on lamp",
            (Self::Lamp, true) => "Turn off lamp",
            (Self::Drawer, false) => "Open drawer",
            (Self::Drawer, true) => "Close drawer",
            (Self::Wardrobe, false) => "Open cupboard / inspect note",
            (Self::Wardrobe, true) => "Close cupboard",
            (Self::Book, false) => "Open book / read clue",
            (Self::Book, true) => "Close book",
            (Self::Apple, _) => "Nudge apple",
            (Self::Painting, false) => "Tilt painting / inspect clue",
            (Self::Painting, true) => "Straighten painting",
            (Self::Sink | Self::Shower, false) => "Turn on tap",
            (Self::Sink | Self::Shower, true) => "Turn off tap",
            (Self::Kettle, false) => "Open kettle spout / release steam",
            (Self::Kettle, true) => "Close kettle spout",
            (Self::Mailbox, false) => "Lower mailbox flag / read letter",
            (Self::Mailbox, true) => "Raise mailbox flag",
            (Self::Doorbell, _) => "Ring doorbell",
            (Self::Dial, _) => "Turn dial / trace machinery",
        }
    }
    pub fn continuous_audio(self) -> bool {
        matches!(
            self,
            Self::Television | Self::Telephone | Self::Sink | Self::Shower | Self::Kettle
        )
    }
    fn loudness(self) -> f32 {
        match self {
            Self::Television => 17.,
            Self::Telephone | Self::Doorbell => 22.,
            Self::Sink | Self::Shower => 11.,
            _ => 3.,
        }
    }
}

#[derive(Component)]
pub struct InteractiveProp {
    pub kind: PropKind,
    pub active: bool,
    pub position: Vec3,
    pub dial: u8,
    rest: Transform,
    amount: f32,
    age: f32,
    noise_in: f32,
    attempt: u32,
}
impl InteractiveProp {
    fn new(kind: PropKind, piece: &Piece) -> Self {
        let on = kind == PropKind::Lamp;
        Self {
            kind,
            active: on,
            position: piece.transform.translation,
            dial: 0,
            rest: Transform::from_translation(piece.transform.translation)
                .with_rotation(piece.transform.rotation),
            amount: u8::from(on) as f32,
            age: 0.,
            noise_in: 0.,
            attempt: 1,
        }
    }
    fn aim(&self) -> Vec3 {
        self.position
            + match self.kind {
                PropKind::Sink | PropKind::Book | PropKind::Apple | PropKind::Kettle => {
                    Vec3::Y * 0.1
                }
                _ => Vec3::Z * 0.12,
            }
    }
    fn activate(&mut self) {
        if self.kind == PropKind::Dial {
            self.dial = (self.dial + 1) % 3;
            self.active = true;
        } else if matches!(
            self.kind,
            PropKind::Telephone | PropKind::Doorbell | PropKind::Apple
        ) {
            self.active = true;
        } else {
            self.active = !self.active;
        }
        self.age = 0.;
        self.noise_in = 0.;
    }
    fn clue(&self) -> &'static str {
        match self.kind {
            PropKind::Television => {
                if self.active {
                    "The TV can lure Peterson into the living room."
                } else {
                    "The TV is off."
                }
            }
            PropKind::Telephone => "The phone will ring for six seconds. Move away now.",
            PropKind::Drawer => {
                "Drawer note: blue for the archive, green for the greenhouse, red for the tower."
            }
            PropKind::Wardrobe => {
                "Cupboard note: follow the windmill shaft to the tower, then the cable to the generator."
            }
            PropKind::Book => {
                let floor = ((self.position.y - FLOOR_Y) / STOREY).floor() as i32;
                match floor {
                    0 => "Note: the blue key was left on the telephone table.",
                    1 => "Note: the workshop crowbar can remove the basement boards.",
                    2 => "Note: the blue archive holds the greenhouse key.",
                    3 => "Note: the station console controls the trolley outside.",
                    _ => {
                        "Note: stop the tower windmill, then cut the generator power to open the freezer."
                    }
                }
            }
            PropKind::Painting => {
                "Behind the frame: three colors, then windmill, generator, and ice. Follow the path to the basement."
            }
            PropKind::Mailbox => "Unsigned letter: why is there machinery above the child's room?",
            PropKind::Dial => match self.dial {
                0 => "Dial / WINDMILL: the shaft above the tower supplies the generator.",
                1 => "Dial / GENERATOR: the red cable leads to the greenhouse on floor 5.",
                _ => "Dial / FREEZER: cut the power before taking the basement keycard.",
            },
            PropKind::Sink | PropKind::Shower => {
                if self.active {
                    "Water is running. The sound can attract Peterson."
                } else {
                    "The tap is off."
                }
            }
            PropKind::Kettle => {
                if self.active {
                    "Steam escapes from the warm kettle spout."
                } else {
                    "The kettle spout is closed."
                }
            }
            PropKind::Doorbell => "The porch bell rings. Peterson may hear it.",
            PropKind::Lamp => {
                if self.active {
                    "The lamp is on."
                } else {
                    "The lamp is off."
                }
            }
            PropKind::Apple => "The apple wobbles on the table.",
        }
    }
}

#[derive(Component)]
struct PropPart {
    owner: Entity,
    rest: Transform,
    role: PartRole,
}
#[derive(Clone, Copy)]
enum PartRole {
    Hinge(f32),
    Cover,
    Effect(u8),
    Screen(u8),
    Bulb,
    Needle,
}
fn part(
    commands: &mut Commands,
    art: &Art,
    owner: Entity,
    position: Vec3,
    size: Vec3,
    paint: Paint,
    role: PartRole,
) {
    let shape = if matches!(role, PartRole::Bulb | PartRole::Effect(_)) {
        Shape::Sphere
    } else {
        Shape::Box
    };
    let id = art.child(commands, owner, position, size, paint, shape);
    commands.entity(id).insert(PropPart {
        owner,
        rest: Transform::from_translation(position).with_scale(size),
        role,
    });
}

pub(crate) fn spawn_piece(commands: &mut Commands, art: &Art, piece: &Piece, seed: usize) -> bool {
    // These surfaces now belong to the animated painting/dial hierarchy.
    if matches!(
        piece.name,
        "Painting canvas" | "Painted apple" | "Signal dial needle"
    ) {
        return true;
    }
    let Some(kind) = PropKind::from_piece(piece.name) else {
        return false;
    };
    let prop = InteractiveProp::new(kind, piece);
    let root = commands
        .spawn((
            Name::new(piece.name),
            prop.rest,
            Visibility::Inherited,
            prop,
        ))
        .id();
    if !matches!(kind, PropKind::Wardrobe | PropKind::Book) {
        art.child(
            commands,
            root,
            Vec3::ZERO,
            piece.transform.scale,
            if kind == PropKind::Television {
                Paint::Black
            } else {
                piece.paint
            },
            piece.shape,
        );
    }
    match kind {
        PropKind::Television => {
            for i in 0..7 {
                part(
                    commands,
                    art,
                    root,
                    Vec3::new(0., -0.29 + i as f32 * 0.09, 0.028),
                    Vec3::new(1., 0.025, 0.005),
                    Paint::Glow,
                    PartRole::Screen(i),
                );
            }
        }
        PropKind::Telephone => {
            art.child(
                commands,
                root,
                Vec3::new(0., 0.15, 0.),
                Vec3::new(0.48, 0.08, 0.1),
                Paint::Black,
                Shape::Box,
            );
            for x in [-0.2, 0.2] {
                art.child(
                    commands,
                    root,
                    Vec3::new(x, 0.11, 0.),
                    Vec3::new(0.09, 0.07, 0.1),
                    Paint::Black,
                    Shape::Sphere,
                );
            }
            for i in 0..6 {
                art.child(
                    commands,
                    root,
                    Vec3::new(0.25, -0.04 - i as f32 * 0.045, 0.03),
                    Vec3::new(0.04, 0.025, 0.07),
                    Paint::Black,
                    Shape::Box,
                );
            }
        }
        PropKind::Lamp => {
            part(
                commands,
                art,
                root,
                Vec3::new(0., -0.15, 0.),
                Vec3::splat(0.1),
                Paint::Glow,
                PartRole::Bulb,
            );
            let lamp = commands
                .spawn((
                    PointLight {
                        intensity: 750.,
                        color: Color::srgb(1., 0.69, 0.37),
                        range: 8.,
                        radius: 0.12,
                        ..default()
                    },
                    Transform::from_xyz(0., -0.25, 0.),
                    Visibility::Inherited,
                    PropPart {
                        owner: root,
                        rest: Transform::from_xyz(0., -0.25, 0.),
                        role: PartRole::Bulb,
                    },
                ))
                .id();
            commands.entity(root).add_child(lamp);
        }
        PropKind::Drawer => {
            art.child(
                commands,
                root,
                Vec3::new(0., -0.06, -0.065),
                Vec3::new(1.1, 0.3, 0.06),
                Paint::Wood,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::new(0., -0.2, -0.35),
                Vec3::new(1.02, 0.04, 0.6),
                Paint::DarkWood,
                Shape::Box,
            );
            for x in [-0.5, 0.5] {
                art.child(
                    commands,
                    root,
                    Vec3::new(x, -0.08, -0.35),
                    Vec3::new(0.04, 0.24, 0.6),
                    Paint::Wood,
                    Shape::Box,
                );
            }
            art.child(
                commands,
                root,
                Vec3::new(0., -0.17, -0.25),
                Vec3::new(0.45, 0.02, 0.23),
                Paint::Cream,
                Shape::Box,
            );
        }
        PropKind::Wardrobe => {
            for side in [-1., 1.] {
                let hinge = commands
                    .spawn((
                        Transform::from_xyz(side * 0.9, 0., 0.),
                        Visibility::Inherited,
                        PropPart {
                            owner: root,
                            rest: Transform::from_xyz(side * 0.9, 0., 0.),
                            role: PartRole::Hinge(side),
                        },
                    ))
                    .id();
                commands.entity(root).add_child(hinge);
                art.child(
                    commands,
                    hinge,
                    Vec3::new(-side * 0.45, 0., 0.),
                    Vec3::new(0.88, 2.3, 0.09),
                    Paint::Wood,
                    Shape::Box,
                );
                art.child(
                    commands,
                    hinge,
                    Vec3::new(-side * 0.77, 0., 0.07),
                    Vec3::new(0.06, 0.21, 0.07),
                    Paint::Metal,
                    Shape::Box,
                );
                art.child(
                    commands,
                    hinge,
                    Vec3::new(-side * 0.45, 0., 0.06),
                    Vec3::new(0.6, 1.9, 0.025),
                    Paint::DarkWood,
                    Shape::Box,
                );
            }
            art.child(
                commands,
                root,
                Vec3::new(0., 0.32, 0.015),
                Vec3::new(0.42, 0.3, 0.015),
                Paint::Cream,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::new(0., -0.5, 0.05),
                Vec3::new(1.6, 0.07, 0.2),
                Paint::Wood,
                Shape::Box,
            );
        }
        PropKind::Book => {
            art.child(
                commands,
                root,
                Vec3::ZERO,
                Vec3::new(0.38, 0.09, 0.56),
                Paint::Cream,
                Shape::Box,
            );
            // The cover is hinged along the spine, rather than through its center.
            let cover = commands
                .spawn((
                    Transform::from_xyz(-0.2, 0.06, 0.),
                    Visibility::Inherited,
                    PropPart {
                        owner: root,
                        rest: Transform::from_xyz(-0.2, 0.06, 0.),
                        role: PartRole::Cover,
                    },
                ))
                .id();
            commands.entity(root).add_child(cover);
            art.child(
                commands,
                cover,
                Vec3::X * 0.2,
                Vec3::new(0.4, 0.025, 0.6),
                Paint::Teal,
                Shape::Box,
            );
            for i in 0..4 {
                art.child(
                    commands,
                    root,
                    Vec3::new(0., 0.052, -0.18 + i as f32 * 0.1),
                    Vec3::new(0.25, 0.005, 0.012),
                    Paint::DarkWood,
                    Shape::Box,
                );
            }
        }
        PropKind::Apple => {
            art.child(
                commands,
                root,
                Vec3::Y * 0.17,
                Vec3::new(0.025, 0.1, 0.025),
                Paint::Trunk,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::new(0.06, 0.19, 0.),
                Vec3::new(0.09, 0.025, 0.035),
                Paint::Leaf,
                Shape::Sphere,
            );
        }
        PropKind::Painting => {
            art.child(
                commands,
                root,
                Vec3::Z * 0.07,
                Vec3::new(1.15, 1.24, 0.03),
                Paint::Cream,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::Z * 0.1,
                Vec3::new(0.4, 0.4, 0.02),
                [Paint::Red, Paint::Teal, Paint::Orange][seed % 3],
                Shape::Sphere,
            );
        }
        PropKind::Sink | PropKind::Shower | PropKind::Kettle => {
            let steam = kind == PropKind::Kettle;
            for i in 0..12 {
                part(
                    commands,
                    art,
                    root,
                    Vec3::ZERO,
                    if steam {
                        Vec3::splat(0.05)
                    } else {
                        Vec3::new(0.012, 0.1, 0.012)
                    },
                    if steam { Paint::White } else { Paint::Glass },
                    PartRole::Effect(i),
                );
            }
            if steam {
                art.child(
                    commands,
                    root,
                    Vec3::new(0.24, 0.1, 0.),
                    Vec3::new(0.22, 0.09, 0.09),
                    Paint::Red,
                    Shape::Cylinder,
                );
                art.child(
                    commands,
                    root,
                    Vec3::new(-0.24, 0.1, 0.),
                    Vec3::new(0.08, 0.35, 0.08),
                    Paint::Black,
                    Shape::Box,
                );
                art.child(
                    commands,
                    root,
                    Vec3::Y * 0.24,
                    Vec3::splat(0.06),
                    Paint::Black,
                    Shape::Sphere,
                );
            }
        }
        PropKind::Dial => {
            part(
                commands,
                art,
                root,
                Vec3::new(0., 0., 0.07),
                Vec3::new(0.035, 0.5, 0.025),
                Paint::Red,
                PartRole::Needle,
            );
            for angle in 0..12 {
                let a = angle as f32 * std::f32::consts::TAU / 12.;
                art.child(
                    commands,
                    root,
                    Vec3::new(a.sin() * 0.3, a.cos() * 0.3, 0.06),
                    Vec3::splat(0.03),
                    Paint::Black,
                    Shape::Box,
                );
            }
        }
        PropKind::Doorbell => {
            art.child(
                commands,
                root,
                Vec3::Z * 0.07,
                Vec3::splat(0.08),
                Paint::Glow,
                Shape::Sphere,
            );
        }
        PropKind::Mailbox => {}
    }
    true
}

fn interact_props(
    keys: Res<ButtonInput<KeyCode>>,
    collision: Res<CollisionWorld>,
    players: Query<&Player>,
    mut session: ResMut<Session>,
    mut props: Query<(Entity, &mut InteractiveProp)>,
    mut neighbors: Query<&mut Neighbor>,
    mut cues: MessageWriter<SoundCue>,
) {
    if session.paused || session.outcome != Outcome::Exploring || !session.prompt.is_empty() {
        return;
    }
    let Ok(player) = players.single() else {
        return;
    };
    let best = props
        .iter()
        .filter(|(_, prop)| can_interact(player, prop.aim(), &collision.solids))
        .min_by(|(_, a), (_, b)| {
            let score = |p: &InteractiveProp| {
                let delta = p.aim() - player.eye();
                (1. - player.forward().dot(delta.normalize_or_zero())) * 5. + delta.length() * 0.1
            };
            score(a).total_cmp(&score(b))
        })
        .map(|(entity, _)| entity);
    let Some(id) = best else {
        return;
    };
    let Ok((_, mut prop)) = props.get_mut(id) else {
        return;
    };
    session.prompt = format!("[E] {}", prop.kind.label(prop.active));
    if keys.just_pressed(KeyCode::KeyE) {
        prop.activate();
        session.tell(prop.clue());
        cues.write(match prop.kind {
            PropKind::Telephone | PropKind::Doorbell => SoundCue::Ring,
            PropKind::Book | PropKind::Painting | PropKind::Mailbox => SoundCue::Read,
            _ => SoundCue::Toggle,
        });
        if let Ok(mut neighbor) = neighbors.single_mut() {
            neighbor.hear(prop.position, prop.kind.loudness(), &collision.solids);
        }
    }
}

type PropPartData<'a> = (
    &'a PropPart,
    &'a mut Transform,
    &'a mut Visibility,
    Option<&'a mut MeshMaterial3d<StandardMaterial>>,
    Option<&'a mut PointLight>,
);

fn animate_props(
    time: Res<Time>,
    session: Res<Session>,
    art: Res<Art>,
    collision: Res<CollisionWorld>,
    mut props: Query<(&mut InteractiveProp, &mut Transform)>,
    mut parts: Query<PropPartData, Without<InteractiveProp>>,
    mut neighbors: Query<&mut Neighbor>,
) {
    if session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    let dt = time.delta_secs();
    for (mut prop, mut tf) in &mut props {
        if prop.attempt != session.attempts {
            prop.active = prop.kind == PropKind::Lamp;
            prop.amount = u8::from(prop.active) as f32;
            prop.age = 0.;
            prop.dial = 0;
            prop.noise_in = 0.;
            prop.attempt = session.attempts;
        }
        prop.age += dt;
        if matches!(
            prop.kind,
            PropKind::Telephone | PropKind::Doorbell | PropKind::Apple
        ) && prop.age
            > if prop.kind == PropKind::Telephone {
                6.
            } else {
                0.8
            }
        {
            prop.active = false;
        }
        prop.amount += (u8::from(prop.active) as f32 - prop.amount) * (1. - (-12. * dt).exp());
        if (f32::from(prop.active) - prop.amount).abs() < 0.0005 {
            prop.amount = f32::from(prop.active);
        }
        let mut target = prop.rest;
        match prop.kind {
            PropKind::Drawer => target.translation.z += prop.amount * 0.55,
            PropKind::Painting => target.rotate_z(prop.amount * 0.28),
            PropKind::Mailbox => target.rotate_x(prop.amount * -1.4),
            PropKind::Telephone => {
                target.translation.x += prop.amount * (prop.age * 40.).sin() * 0.012;
            }
            PropKind::Doorbell => target.translation.z -= prop.amount * 0.025,
            PropKind::Apple => {
                target
                    .rotate_z((prop.age * 18.).sin() * (-prop.age * 4.).exp() * 0.4 * prop.amount);
            }
            _ => {}
        }
        tf.set_if_neq(target);
        prop.noise_in -= dt;
        if prop.active && prop.kind.continuous_audio() && prop.noise_in <= 0. {
            if let Ok(mut neighbor) = neighbors.single_mut() {
                neighbor.hear(prop.position, prop.kind.loudness(), &collision.solids);
            }
            prop.noise_in = 1.2;
        }
    }
    for (part, mut tf, mut visible, material, light) in &mut parts {
        let Ok((prop, _)) = props.get(part.owner) else {
            continue;
        };
        if matches!(part.role, PartRole::Screen(_) | PartRole::Effect(_)) && !prop.active {
            visible.set_if_neq(Visibility::Hidden);
            continue;
        }
        let mut target = part.rest;
        match part.role {
            PartRole::Hinge(side) => target.rotate_y(side * prop.amount * 1.65),
            PartRole::Cover => target.rotate_z(prop.amount * 2.6),
            PartRole::Needle => target.rotate_z(-0.9 + prop.dial as f32 * 0.9),
            PartRole::Bulb => {
                if let Some(mut mat) = material {
                    let handle = art.material(if prop.active {
                        Paint::Glow
                    } else {
                        Paint::Cream
                    });
                    if mat.0 != handle {
                        mat.0 = handle;
                    }
                }
                if let Some(mut lamp) = light {
                    let intensity = 750. * prop.amount;
                    if lamp.intensity != intensity {
                        lamp.intensity = intensity;
                    }
                }
            }
            PartRole::Screen(i) => {
                visible.set_if_neq(Visibility::Inherited);
                target.translation.y = -0.3 + ((prop.age * 0.14 + i as f32 * 0.09).rem_euclid(0.6));
                target.scale.x *= 0.85 + 0.1 * (prop.age * 9. + i as f32).sin();
            }
            PartRole::Effect(i) => {
                visible.set_if_neq(Visibility::Inherited);
                let t = (prop.age * 1.5 + i as f32 / 12.).fract();
                target.translation = match prop.kind {
                    PropKind::Kettle => Vec3::new(
                        0.3 + (t * 8. + i as f32).sin() * t * 0.07,
                        0.16 + t * 0.7,
                        t * 0.08,
                    ),
                    PropKind::Shower => Vec3::new(
                        -0.55 + ((i * 7) % 5) as f32 * 0.04,
                        1.15 - t * 1.6,
                        0.03 + (i % 3) as f32 * 0.06,
                    ),
                    _ => Vec3::new((i as f32).sin() * 0.02, 0.5 - t * 0.47, -0.05),
                };
                if prop.kind == PropKind::Kettle {
                    target.scale *= 0.4 + t * 1.7;
                }
            }
        }
        tf.set_if_neq(target);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f32(1. / 60.),
            ))
            .insert_resource(Art::test_fixture())
            .init_resource::<Session>()
            .init_resource::<crate::world::HousePlan>()
            .init_resource::<CollisionWorld>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<SoundCue>()
            .add_systems(
                Startup,
                |mut commands: Commands,
                 art: Res<Art>,
                 plan: Res<crate::world::HousePlan>,
                 mut collision: ResMut<CollisionWorld>| {
                    collision.solids = plan.colliders.clone();
                    for (index, piece) in plan.pieces.iter().enumerate() {
                        spawn_piece(&mut commands, &art, piece, index);
                    }
                    commands.spawn(Player::new(crate::world::PLAYER_SPAWN));
                    commands.spawn(Neighbor::default());
                },
            )
            .add_systems(
                Update,
                (|mut session: ResMut<Session>| session.prompt.clear()).in_set(GameSet::Interact),
            )
            .add_plugins(PropsPlugin);
        app.update();
        app.update();
        app
    }
    fn aim_at(player: &mut Player, point: Vec3) {
        let direction = (point - player.eye()).normalize();
        player.yaw = (-direction.x).atan2(-direction.z);
        player.pitch = direction.y.asin();
    }
    #[test]
    fn every_household_target_has_a_clear_approach_without_standing_in_furniture() {
        let plan = crate::world::HousePlan::default();
        for piece in &plan.pieces {
            let Some(kind) = PropKind::from_piece(piece.name) else {
                continue;
            };
            let prop = InteractiveProp::new(kind, piece);
            let floor = FLOOR_Y + 0.002 + ((prop.position.y - FLOOR_Y) / STOREY).floor() * STOREY;
            let mut reachable = false;
            for radius in [0.8, 1.3, 1.8, 2.2] {
                for i in 0..32 {
                    let angle = i as f32 * std::f32::consts::TAU / 32.;
                    let mut p = Player::new(Vec3::new(
                        prop.position.x + angle.sin() * radius,
                        floor,
                        prop.position.z + angle.cos() * radius,
                    ));
                    aim_at(&mut p, prop.aim());
                    if !plan.colliders.iter().any(|c| p.aabb().overlaps(c))
                        && can_interact(&p, prop.aim(), &plan.colliders)
                    {
                        reachable = true;
                    }
                }
            }
            assert!(
                reachable,
                "{} {:?} has no reachable interaction at {:?}",
                piece.name, kind, piece.transform.translation
            );
        }
    }
    #[test]
    fn every_authored_household_object_responds_to_the_actual_e_handler() {
        let mut app = app();
        let plan = crate::world::HousePlan::default();
        let player_id = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        let ids: Vec<_> = app
            .world_mut()
            .query_filtered::<Entity, With<InteractiveProp>>()
            .iter(app.world())
            .collect();
        for id in ids {
            let prop = app.world().get::<InteractiveProp>(id).unwrap();
            let kind = prop.kind;
            let target = prop.aim();
            let position = prop.position;
            let before = prop.active;
            let floor = FLOOR_Y + 0.002 + ((position.y - FLOOR_Y) / STOREY).floor() * STOREY;
            let mut approach = None;
            for radius in [0.8, 1.3, 1.8, 2.2] {
                for i in 0..32 {
                    let angle = i as f32 * std::f32::consts::TAU / 32.;
                    let mut p = Player::new(Vec3::new(
                        position.x + angle.sin() * radius,
                        floor,
                        position.z + angle.cos() * radius,
                    ));
                    aim_at(&mut p, target);
                    if !plan.colliders.iter().any(|c| p.aabb().overlaps(c))
                        && can_interact(&p, target, &plan.colliders)
                    {
                        approach = Some(p);
                        break;
                    }
                }
                if approach.is_some() {
                    break;
                }
            }
            *app.world_mut().get_mut::<Player>(player_id).unwrap() =
                approach.expect("clear authored approach");
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyE);
            app.update();
            let prop = app.world().get::<InteractiveProp>(id).unwrap();
            if matches!(
                kind,
                PropKind::Telephone | PropKind::Doorbell | PropKind::Apple | PropKind::Dial
            ) {
                assert!(prop.active, "{kind:?} did not activate at {position:?}");
            } else {
                assert_ne!(
                    prop.active, before,
                    "{kind:?} did not toggle at {position:?}"
                );
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
        }
    }
    #[test]
    fn real_ecs_tv_attracts_neighbor_and_paused_props_freeze_then_retry_resets() {
        let mut app = app();
        let tv = app
            .world_mut()
            .query::<(Entity, &InteractiveProp)>()
            .iter(app.world())
            .find(|(_, p)| p.kind == PropKind::Television)
            .map(|(e, _)| e)
            .unwrap();
        let target = app.world().get::<InteractiveProp>(tv).unwrap().aim();
        let player_id = app
            .world_mut()
            .query_filtered::<Entity, With<Player>>()
            .single(app.world())
            .unwrap();
        let mut p = app.world_mut().get_mut::<Player>(player_id).unwrap();
        p.pos = Vec3::new(-8., FLOOR_Y, 2.5);
        aim_at(&mut p, target);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        assert!(app.world().get::<InteractiveProp>(tv).unwrap().active);
        assert_eq!(
            app.world_mut()
                .query::<&Neighbor>()
                .single(app.world())
                .unwrap()
                .state,
            crate::ai::AlertState::Investigate
        );
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        app.world_mut().resource_mut::<Session>().paused = true;
        let age = app.world().get::<InteractiveProp>(tv).unwrap().age;
        for _ in 0..30 {
            app.update();
        }
        assert_eq!(app.world().get::<InteractiveProp>(tv).unwrap().age, age);
        app.world_mut().resource_mut::<Session>().paused = false;
        app.world_mut().resource_mut::<Session>().attempts += 1;
        app.update();
        assert!(!app.world().get::<InteractiveProp>(tv).unwrap().active);
        assert!(
            app.world_mut()
                .query::<&InteractiveProp>()
                .iter(app.world())
                .filter(|p| p.kind == PropKind::Lamp)
                .all(|p| p.active)
        );
    }
    #[test]
    fn water_particles_are_bounded_and_door_animation_settles_without_drift() {
        let mut app = app();
        let ids: Vec<_> = app
            .world_mut()
            .query::<(Entity, &InteractiveProp)>()
            .iter(app.world())
            .map(|(e, _)| e)
            .collect();
        let count = app.world().entities().len();
        for &id in &ids {
            app.world_mut()
                .get_mut::<InteractiveProp>(id)
                .unwrap()
                .activate();
        }
        for _ in 0..120 {
            app.update();
        }
        assert_eq!(app.world().entities().len(), count);
        for &id in &ids {
            app.world_mut()
                .get_mut::<InteractiveProp>(id)
                .unwrap()
                .active = false;
        }
        for _ in 0..120 {
            app.update();
        }
        for &id in &ids {
            let p = app.world().get::<InteractiveProp>(id).unwrap();
            assert!(p.amount < 0.001, "{:?} failed to settle", p.kind);
            let tf = app.world().get::<Transform>(id).unwrap();
            assert!(tf.translation.distance(p.rest.translation) < 0.001);
        }
    }
    #[test]
    fn interactions_change_state_and_timed_distractions_can_be_retriggered() {
        let piece = Piece {
            name: "Telephone",
            transform: Transform::IDENTITY,
            paint: Paint::Black,
            shape: Shape::Box,
        };
        let mut phone = InteractiveProp::new(PropKind::Telephone, &piece);
        phone.activate();
        phone.age = 5.;
        phone.activate();
        assert!(phone.active);
        assert_eq!(phone.age, 0.);
        let mut lamp = InteractiveProp::new(PropKind::Lamp, &piece);
        assert!(lamp.active);
        lamp.activate();
        assert!(!lamp.active);
        lamp.activate();
        assert!(lamp.active);
        let mut dial = InteractiveProp::new(PropKind::Dial, &piece);
        for expected in [1, 2, 0] {
            dial.activate();
            assert_eq!(dial.dial, expected);
        }
    }
    #[test]
    fn authored_house_has_interactions_across_all_main_floors() {
        let plan = crate::world::HousePlan::default();
        let interactive: Vec<_> = plan
            .pieces
            .iter()
            .filter(|p| PropKind::from_piece(p.name).is_some())
            .collect();
        assert!(
            interactive.len() >= 35,
            "{} interactive household objects",
            interactive.len()
        );
        for name in [
            "CRT screen",
            "Telephone",
            "Sink",
            "Shower valve",
            "Dresser drawer",
            "Wardrobe doors",
            "Signal dial",
            "Porch doorbell",
        ] {
            assert!(interactive.iter().any(|p| p.name == name), "missing {name}");
        }
    }
}
