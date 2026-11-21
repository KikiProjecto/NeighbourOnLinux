//! Deterministic underground layout and its circuit / service-exit progression.
use crate::{
    ai::Neighbor,
    audio::SoundCue,
    game::{Art, GameSet, Outcome, Session, can_interact},
    items::PuzzleProgress,
    player::Player,
    world::{CollisionWorld, Paint, Shape},
};
use bevy::{ecs::system::SystemParam, prelude::*};
use std::collections::VecDeque;

mod map;
mod signs;

pub const WIDTH: usize = 11;
pub const CELL: f32 = 4.4;
pub const FLOOR: f32 = -4.2;
pub const HEIGHT: f32 = 3.2;
pub const ENTRY: (usize, usize) = (5, 5);
pub const STAIR_CELL: (usize, usize) = (5, 6);
pub const DIRECTIONS: [(i32, i32); 4] = [(0, -1), (1, 0), (0, 1), (-1, 0)];

#[derive(Clone, Copy)]
pub struct RoomSpec {
    pub name: &'static str,
    pub x: usize,
    pub z: usize,
}
impl RoomSpec {
    pub fn center(self) -> Vec3 {
        cell_center(self.x, self.z) + Vec3::new(CELL * 0.5, 0., CELL * 0.5)
    }
}
pub const ROOMS: [RoomSpec; 12] = [
    RoomSpec {
        name: "Underground · old boiler room",
        x: 0,
        z: 0,
    },
    RoomSpec {
        name: "Underground · maintenance workshop",
        x: 3,
        z: 0,
    },
    RoomSpec {
        name: "Underground · pump station",
        x: 6,
        z: 0,
    },
    RoomSpec {
        name: "Underground · generator vault",
        x: 9,
        z: 0,
    },
    RoomSpec {
        name: "Underground · abandoned cistern",
        x: 0,
        z: 3,
    },
    RoomSpec {
        name: "Underground · cold storage",
        x: 9,
        z: 3,
    },
    RoomSpec {
        name: "Underground · records archive",
        x: 0,
        z: 6,
    },
    RoomSpec {
        name: "Underground · observation room",
        x: 9,
        z: 6,
    },
    RoomSpec {
        name: "Underground · ventilation chamber",
        x: 0,
        z: 9,
    },
    RoomSpec {
        name: "Underground · improvised laboratory",
        x: 3,
        z: 9,
    },
    RoomSpec {
        name: "Underground · evidence vault",
        x: 6,
        z: 9,
    },
    RoomSpec {
        name: "Underground · service exit",
        x: 9,
        z: 9,
    },
];
pub fn cell_center(x: usize, z: usize) -> Vec3 {
    Vec3::new(
        -8. + (x as f32 - 5.) * CELL,
        FLOOR,
        -10.8 + (z as f32 - 5.) * CELL,
    )
}

#[derive(Clone)]
pub struct Labyrinth {
    pub passages: [[u8; WIDTH]; WIDTH],
}
impl Default for Labyrinth {
    fn default() -> Self {
        let mut maze = Self {
            passages: [[0; WIDTH]; WIDTH],
        };
        let mut visited = [[false; WIDTH]; WIDTH];
        let mut stack = vec![ENTRY];
        let mut rng = 0x6e65_6967_u32;
        visited[ENTRY.1][ENTRY.0] = true;
        // Reserve this cell for the staircase; it is a leaf off the maze.
        visited[STAIR_CELL.1][STAIR_CELL.0] = true;
        while let Some(&(x, z)) = stack.last() {
            let options: Vec<_> = (0..4)
                .filter_map(|direction| {
                    maze.neighbor(x, z, direction)
                        .filter(|&(nx, nz)| !visited[nz][nx])
                        .map(|cell| (direction, cell))
                })
                .collect();
            if options.is_empty() {
                stack.pop();
                continue;
            }
            rng = rng.wrapping_mul(1664525).wrapping_add(1013904223);
            let (direction, (nx, nz)) = options[rng as usize % options.len()];
            maze.open(x, z, direction);
            visited[nz][nx] = true;
            stack.push((nx, nz));
        }
        for room in ROOMS {
            for z in room.z..room.z + 2 {
                maze.open(room.x, z, 1);
            }
            for x in room.x..room.x + 2 {
                maze.open(x, room.z, 2);
            }
        }
        maze.open(ENTRY.0, ENTRY.1, 2); // Clearance for the descending entrance stairs.
        for z in [2, 5, 8] {
            for x in [2, 5, 8] {
                maze.open(x, z, 1);
            }
        }
        maze
    }
}
impl Labyrinth {
    pub fn neighbor(&self, x: usize, z: usize, direction: usize) -> Option<(usize, usize)> {
        let (dx, dz) = DIRECTIONS[direction];
        let nx = x as i32 + dx;
        let nz = z as i32 + dz;
        ((0..WIDTH as i32).contains(&nx) && (0..WIDTH as i32).contains(&nz))
            .then_some((nx as usize, nz as usize))
    }
    fn open(&mut self, x: usize, z: usize, direction: usize) {
        if let Some((nx, nz)) = self.neighbor(x, z, direction) {
            self.passages[z][x] |= 1 << direction;
            self.passages[nz][nx] |= 1 << ((direction + 2) % 4);
        }
    }
    pub fn path(&self, destination: (usize, usize)) -> Vec<(usize, usize)> {
        let mut previous = [[None; WIDTH]; WIDTH];
        previous[ENTRY.1][ENTRY.0] = Some(ENTRY);
        let mut queue = VecDeque::from([ENTRY]);
        while let Some((x, z)) = queue.pop_front() {
            if (x, z) == destination {
                break;
            }
            for direction in 0..4 {
                if self.passages[z][x] & (1 << direction) == 0 {
                    continue;
                }
                if let Some((nx, nz)) = self.neighbor(x, z, direction)
                    && previous[nz][nx].is_none()
                {
                    previous[nz][nx] = Some((x, z));
                    queue.push_back((nx, nz));
                }
            }
        }
        let mut cursor = destination;
        let mut path = vec![cursor];
        while cursor != ENTRY {
            let Some(parent) = previous[cursor.1][cursor.0] else {
                return vec![];
            };
            cursor = parent;
            path.push(cursor);
        }
        path.reverse();
        path
    }
}

pub const CIRCUIT_ROOMS: [usize; 3] = [0, 3, 8];
pub fn circuit_position(index: usize) -> Vec3 {
    ROOMS[CIRCUIT_ROOMS[index]].center() + Vec3::new(0., 1.2, -3.7)
}
pub fn key_position() -> Vec3 {
    ROOMS[10].center() + Vec3::new(0., 1.15, 0.)
}
pub fn exit_position() -> Vec3 {
    ROOMS[11].center() + Vec3::new(0., 1.35, -3.65)
}
pub fn map_position() -> Vec3 {
    cell_center(ENTRY.0, ENTRY.1) + Vec3::new(-1.65, 1.25, 0.9)
}
pub fn notes_position() -> Vec3 {
    ROOMS[6].center() + Vec3::new(0., 1.1, 0.)
}

#[derive(Resource)]
pub struct BasementProgress {
    pub circuits: [bool; 3],
    pub service_key: bool,
    pub escaped: bool,
    attempt: u32,
    pub map_open: bool,
    pub map_known: bool,
}
impl Default for BasementProgress {
    fn default() -> Self {
        Self {
            circuits: [false; 3],
            service_key: false,
            escaped: false,
            attempt: 1,
            map_open: false,
            map_known: false,
        }
    }
}
impl BasementProgress {
    pub fn powered(&self) -> bool {
        self.circuits.iter().all(|on| *on)
    }
    pub fn objective(&self) -> String {
        if self.escaped {
            "You escaped the underground labyrinth.".into()
        } else if !self.powered() {
            format!(
                "UNDERGROUND / Restore three circuits ({}/3). Follow the colored cables.",
                self.circuits.iter().filter(|on| **on).count()
            )
        } else if !self.service_key {
            "UNDERGROUND / Find the service key in the evidence vault.".into()
        } else {
            "UNDERGROUND / Take the service key to the service exit.".into()
        }
    }
}
#[derive(Clone, Copy)]
enum Control {
    Circuit(usize),
    Key,
    Exit,
    Map,
    Notes,
}
impl Control {
    fn position(self) -> Vec3 {
        match self {
            Self::Circuit(i) => circuit_position(i),
            Self::Key => key_position(),
            Self::Exit => exit_position(),
            Self::Map => map_position(),
            Self::Notes => notes_position(),
        }
    }
}
const CONTROLS: [Control; 7] = [
    Control::Circuit(0),
    Control::Circuit(1),
    Control::Circuit(2),
    Control::Key,
    Control::Exit,
    Control::Map,
    Control::Notes,
];
#[derive(Component)]
struct Lever(usize);
#[derive(Component)]
struct Indicator(usize);
#[derive(Component)]
struct ServiceKey;
#[derive(Component)]
struct VentilationFan;
#[derive(Component)]
struct KeyCaseLid;
#[derive(Component)]
pub(crate) struct UndergroundMesh {
    pub center: Vec3,
    pub radius: f32,
}
#[derive(Component)]
pub(crate) struct UndergroundLamp(pub usize);

pub struct BasementPlugin;
impl Plugin for BasementPlugin {
    fn build(&self, app: &mut App) {
        map::install(app);
        app.init_resource::<BasementProgress>()
            .add_systems(Startup, (setup, signs::setup).after(GameSet::Setup))
            .add_systems(
                Update,
                controls
                    .after(GameSet::Interact)
                    .before(GameSet::Presentation),
            )
            .add_systems(
                Update,
                present.after(controls).before(GameSet::Presentation),
            )
            .add_systems(Update, cull_underground.after(GameSet::Presentation));
    }
}
fn setup(
    mut commands: Commands,
    art: Res<Art>,
    options: Res<crate::LaunchOptions>,
    mut progress: ResMut<BasementProgress>,
    mut gate: ResMut<PuzzleProgress>,
    mut session: ResMut<Session>,
) {
    if options.showcase
        && matches!(
            options.inspect,
            Some(
                crate::Inspection::Basement
                    | crate::Inspection::BasementLab
                    | crate::Inspection::BasementVault
            )
        )
    {
        gate.basement_open = true;
        session.tell("Follow the colored circuits. The service key opens the underground exit.");
        progress.circuits = [true; 3];
        progress.map_known = true;
        progress.map_open = options.controls;
    }
    for (index, room) in ROOMS.into_iter().enumerate() {
        commands.spawn((
            Name::new("Underground chamber light"),
            UndergroundLamp(index % 3),
            PointLight {
                color: match index % 3 {
                    0 => Color::srgb(1., 0.63, 0.32),
                    1 => Color::srgb(0.4, 0.76, 0.9),
                    _ => Color::srgb(0.71, 0.6, 0.9),
                },
                intensity: 2400.,
                range: 10.,
                radius: 0.22,
                ..default()
            },
            Transform::from_translation(room.center() + Vec3::Y * 2.85),
        ));
    }
    for z in (1..WIDTH).step_by(3) {
        for x in (1..WIDTH).step_by(3) {
            commands.spawn((
                Name::new("Underground tunnel light"),
                UndergroundLamp((x + z) % 3),
                PointLight {
                    color: Color::srgb(0.95, 0.73, 0.44),
                    intensity: 2400.,
                    range: 7.,
                    radius: 0.15,
                    ..default()
                },
                Transform::from_translation(cell_center(x, z) + Vec3::Y * 2.8),
            ));
        }
    }
    for index in 0..3 {
        let root = commands
            .spawn((
                Name::new("Basement breaker lever"),
                Transform::from_translation(circuit_position(index) + Vec3::new(0., 0., 0.17)),
                Visibility::Inherited,
                Lever(index),
            ))
            .id();
        art.child(
            &mut commands,
            root,
            Vec3::ZERO,
            Vec3::new(0.1, 0.58, 0.12),
            Paint::Metal,
            Shape::Box,
        );
        art.child(
            &mut commands,
            root,
            Vec3::new(0., 0.25, 0.04),
            Vec3::new(0.37, 0.13, 0.13),
            Paint::Red,
            Shape::Box,
        );
        let indicator = art.spawn(
            &mut commands,
            &crate::world::Piece {
                name: "Circuit status",
                transform: Transform::from_translation(
                    circuit_position(index) + Vec3::new(0.3, 0.35, 0.14),
                )
                .with_scale(Vec3::new(0.12, 0.12, 0.07)),
                paint: Paint::Red,
                shape: Shape::Sphere,
            },
        );
        commands.entity(indicator).insert(Indicator(index));
    }
    let fan = commands
        .spawn((
            Name::new("Basement ventilation fan"),
            VentilationFan,
            Transform::from_translation(ROOMS[8].center() + Vec3::new(0., 2.15, 3.78)),
            Visibility::Inherited,
        ))
        .id();
    for angle in [0., std::f32::consts::FRAC_PI_2] {
        let blade = art.child(
            &mut commands,
            fan,
            Vec3::ZERO,
            Vec3::new(1.3, 0.14, 0.08),
            Paint::Metal,
            Shape::Box,
        );
        commands.entity(blade).insert(
            Transform::from_rotation(Quat::from_rotation_z(angle))
                .with_scale(Vec3::new(1.3, 0.14, 0.08)),
        );
    }
    let lid = commands
        .spawn((
            Name::new("Powered service key case lid"),
            KeyCaseLid,
            Transform::from_translation(key_position() + Vec3::new(0., -0.06, -0.37)),
            Visibility::Inherited,
        ))
        .id();
    for x in [-0.44, 0.44] {
        art.child(
            &mut commands,
            lid,
            Vec3::new(x, 0.28, 0.37),
            Vec3::new(0.035, 0.045, 0.74),
            Paint::Metal,
            Shape::Box,
        );
    }
    for z in [0., 0.74] {
        art.child(
            &mut commands,
            lid,
            Vec3::new(0., 0.28, z),
            Vec3::new(0.9, 0.045, 0.035),
            Paint::Metal,
            Shape::Box,
        );
    }
    for x in [-0.3, 0., 0.3] {
        art.child(
            &mut commands,
            lid,
            Vec3::new(x, 0.28, 0.37),
            Vec3::new(0.025, 0.035, 0.74),
            Paint::Metal,
            Shape::Box,
        );
    }
    let key = commands
        .spawn((
            Name::new("Basement service key"),
            ServiceKey,
            Transform::from_translation(key_position()),
            Visibility::Inherited,
        ))
        .id();
    art.child(
        &mut commands,
        key,
        Vec3::new(-0.14, 0., 0.),
        Vec3::new(0.13, 0.13, 0.04),
        Paint::Yellow,
        Shape::Sphere,
    );
    art.child(
        &mut commands,
        key,
        Vec3::new(0.08, 0., 0.),
        Vec3::new(0.36, 0.055, 0.055),
        Paint::Yellow,
        Shape::Box,
    );
    for x in [0.14, 0.23] {
        art.child(
            &mut commands,
            key,
            Vec3::new(x, -0.055, 0.),
            Vec3::new(0.04, 0.12, 0.055),
            Paint::Yellow,
            Shape::Box,
        );
    }
}
#[derive(SystemParam)]
struct BasementWorld<'w> {
    gate: Res<'w, PuzzleProgress>,
    collision: Res<'w, CollisionWorld>,
}
fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<Session>,
    mut progress: ResMut<BasementProgress>,
    world: BasementWorld,
    players: Query<&Player>,
    mut neighbors: Query<&mut Neighbor>,
    mut sound: MessageWriter<SoundCue>,
) {
    if progress.attempt != session.attempts {
        *progress = BasementProgress {
            attempt: session.attempts,
            ..default()
        };
    }
    if session.paused || session.outcome != Outcome::Exploring || !world.gate.basement_open {
        return;
    }
    let Ok(player) = players.single() else {
        return;
    };
    if keys.just_pressed(KeyCode::Tab) && progress.map_known {
        progress.map_open = !progress.map_open;
    }
    let Some(control) = CONTROLS
        .into_iter()
        .find(|control| can_interact(player, control.position(), &world.collision.solids))
    else {
        return;
    };
    if matches!(control, Control::Key) && progress.service_key {
        return;
    }
    session.prompt = match control {
        Control::Circuit(i) => format!(
            "[E] {} circuit {}",
            if progress.circuits[i] {
                "Inspect restored"
            } else {
                "Restore"
            },
            i + 1
        ),
        Control::Key => "[E] Open the powered key case".into(),
        Control::Exit => "[E] Unlock the service exit".into(),
        Control::Map => "[E] Read the maintenance route board".into(),
        Control::Notes => "[E] Read the underground records".into(),
    };
    if !keys.just_pressed(KeyCode::KeyE) {
        return;
    }
    let mut cue = SoundCue::Read;
    match control {
        Control::Circuit(i) => {
            progress.circuits[i] = true;
            cue = SoundCue::Toggle;
            session.tell(if progress.powered(){"All three circuits are live. The key case in the evidence vault is unlocked.".into()}else{format!("Circuit {} restored. Follow the other colored cables to the remaining breakers.",i+1)});
        }
        Control::Key => {
            if progress.powered() {
                progress.service_key = true;
                cue = SoundCue::Pickup;
                session
                    .tell("Service key collected. Find the service exit in the southeast chamber.");
            } else {
                session
                    .tell("The key case has no power. Restore all three basement circuits first.");
            }
        }
        Control::Exit => {
            if progress.powered() && progress.service_key {
                progress.escaped = true;
                session.outcome = Outcome::Escaped;
                cue = SoundCue::Win;
                session.tell("You escaped through the underground service tunnel.");
            } else {
                session.tell(
                    "The service exit needs restored power and the key from the evidence vault.",
                );
            }
        }
        Control::Map => {
            progress.map_known = true;
            progress.map_open = !progress.map_open;
            session.tell("Maintenance map copied. TAB opens the map. Follow the three colored cables to the breakers.");
        }
        Control::Notes => session
            .tell("The evidence vault holds the service key. Three circuits release its case."),
    }
    sound.write(cue);
    if let Ok(mut neighbor) = neighbors.single_mut() {
        neighbor.hear(player.pos, 14., &world.collision.solids);
    }
}
#[derive(SystemParam)]
struct BasementPresentation<'w> {
    time: Res<'w, Time>,
    session: Res<'w, Session>,
    progress: Res<'w, BasementProgress>,
    art: Res<'w, Art>,
}
type MechanismFilter = (
    Or<(With<VentilationFan>, With<KeyCaseLid>)>,
    Without<Lever>,
    Without<ServiceKey>,
);
type LampFilter = (With<UndergroundLamp>, Without<UndergroundMesh>);
fn present(
    state: BasementPresentation,
    mut levers: Query<(&Lever, &mut Transform), Without<ServiceKey>>,
    mut keys: Query<(&mut Visibility, &mut Transform), With<ServiceKey>>,
    mut lamps: Query<(&Indicator, &mut MeshMaterial3d<StandardMaterial>)>,
    mut lights: Query<(&UndergroundLamp, &mut PointLight)>,
    mut mechanisms: Query<(&mut Transform, Has<VentilationFan>), MechanismFilter>,
) {
    let BasementPresentation {
        time,
        session,
        progress,
        art,
    } = state;
    if session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    for (mut transform, is_fan) in &mut mechanisms {
        if is_fan {
            if progress.circuits[2] {
                transform.rotate_z(time.delta_secs() * 5.);
            }
        } else {
            let wanted = Quat::from_rotation_x(if progress.powered() { -1.6 } else { 0. });
            let rotation = transform
                .rotation
                .slerp(wanted, 1. - (-6. * time.delta_secs()).exp());
            if transform.rotation.angle_between(wanted) > 0.001 {
                transform.rotation = rotation;
            } else if transform.rotation != wanted {
                transform.rotation = wanted;
            }
        }
    }
    for (lever, mut transform) in &mut levers {
        let wanted = Quat::from_rotation_z(if progress.circuits[lever.0] {
            -0.9
        } else {
            0.65
        });
        let rotation = transform
            .rotation
            .slerp(wanted, 1. - (-10. * time.delta_secs()).exp());
        if transform.rotation != rotation {
            transform.rotation = rotation;
        }
    }
    for (mut visible, mut transform) in &mut keys {
        visible.set_if_neq(if progress.service_key {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        transform.rotate_y(time.delta_secs() * 0.6);
    }
    for (indicator, mut material) in &mut lamps {
        let wanted = art.material(if progress.circuits[indicator.0] {
            Paint::Glow
        } else {
            Paint::Red
        });
        if material.0 != wanted {
            material.0 = wanted;
        }
    }
    for (lamp, mut light) in &mut lights {
        let intensity = if progress.circuits[lamp.0] {
            8000.
        } else {
            2400.
        };
        if light.intensity != intensity {
            light.intensity = intensity;
        }
    }
}

fn cull_underground(
    gate: Res<PuzzleProgress>,
    players: Query<&Player>,
    mut geometry: Query<(&UndergroundMesh, &mut Visibility)>,
    mut lights: Query<(&GlobalTransform, &mut Visibility), LampFilter>,
) {
    let Ok(player) = players.single() else {
        return;
    };
    let viewing = player.pos.y < -0.4
        || gate.basement_open && player.pos.distance(Vec3::new(-8., 0., -5.)) < 14.;
    for (mesh, mut visibility) in &mut geometry {
        let visible =
            viewing && player.eye().distance_squared(mesh.center) < (30. + mesh.radius).powi(2);
        visibility.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (transform, mut visibility) in &mut lights {
        let visible =
            viewing && player.eye().distance_squared(transform.translation()) < 18_f32.powi(2);
        visibility.set_if_neq(if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn maze_is_reproducible_connected_and_has_bidirectional_passages() {
        let maze = Labyrinth::default();
        assert_eq!(maze.passages, Labyrinth::default().passages);
        for z in 0..WIDTH {
            for x in 0..WIDTH {
                assert!(!maze.path((x, z)).is_empty());
                for direction in 0..4 {
                    if maze.passages[z][x] & (1 << direction) != 0 {
                        let (nx, nz) = maze.neighbor(x, z, direction).unwrap();
                        assert_ne!(maze.passages[nz][nx] & (1 << ((direction + 2) % 4)), 0);
                    }
                }
            }
        }
        assert!(maze.path((0, 0)).len() > 10);
    }
    fn activate(app: &mut App, position: Vec3) {
        let mut query = app.world_mut().query::<&mut Player>();
        {
            let mut player = query.single_mut(app.world_mut()).unwrap();
            let plan = crate::world::HousePlan::default();
            player.pos = (0..16)
                .find_map(|index| {
                    let angle = index as f32 * std::f32::consts::TAU / 16.;
                    let feet = position.with_y(FLOOR)
                        + Vec3::new(angle.cos() * 1.8, 0., angle.sin() * 1.8);
                    let mut candidate = Player::new(feet);
                    let direction = (position - candidate.eye()).normalize();
                    candidate.yaw = (-direction.x).atan2(-direction.z);
                    candidate.pitch = direction.y.asin();
                    if plan
                        .colliders
                        .iter()
                        .any(|solid| candidate.aabb().overlaps(solid))
                        || !can_interact(&candidate, position, &plan.colliders)
                    {
                        None
                    } else {
                        Some(feet)
                    }
                })
                .expect("control has a clear approach");
            let direction = (position - player.eye()).normalize();
            player.yaw = (-direction.x).atan2(-direction.z);
            player.pitch = direction.y.asin();
        }
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::KeyE);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
    }
    #[test]
    fn real_e_handler_requires_three_circuits_then_key_then_exit_and_retry_resets() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ButtonInput<KeyCode>>()
            .init_resource::<Session>()
            .init_resource::<BasementProgress>()
            .insert_resource(CollisionWorld {
                solids: crate::world::HousePlan::default().colliders,
                ..default()
            })
            .insert_resource(PuzzleProgress {
                basement_open: true,
                ..default()
            })
            .add_message::<SoundCue>()
            .add_systems(Update, controls);
        app.world_mut().spawn(Player::new(cell_center(5, 5)));
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        assert!(!app.world().resource::<BasementProgress>().map_open);
        activate(&mut app, map_position());
        assert!(app.world().resource::<BasementProgress>().map_known);
        assert!(app.world().resource::<BasementProgress>().map_open);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(KeyCode::Tab);
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
        assert!(!app.world().resource::<BasementProgress>().map_open);
        app.world_mut().resource_mut::<Session>().paused = true;
        activate(&mut app, circuit_position(0));
        assert!(!app.world().resource::<BasementProgress>().circuits[0]);
        app.world_mut().resource_mut::<Session>().paused = false;
        activate(&mut app, key_position());
        assert!(!app.world().resource::<BasementProgress>().service_key);
        activate(&mut app, exit_position());
        assert!(app.world().resource::<Session>().outcome == Outcome::Exploring);
        for index in [2, 0, 1] {
            activate(&mut app, circuit_position(index));
        }
        assert!(app.world().resource::<BasementProgress>().powered());
        activate(&mut app, exit_position());
        assert!(app.world().resource::<Session>().outcome == Outcome::Exploring);
        activate(&mut app, key_position());
        assert!(app.world().resource::<BasementProgress>().service_key);
        activate(&mut app, exit_position());
        assert!(app.world().resource::<Session>().outcome == Outcome::Escaped);
        {
            let mut session = app.world_mut().resource_mut::<Session>();
            session.attempts += 1;
            session.outcome = Outcome::Exploring;
        }
        app.update();
        let progress = app.world().resource::<BasementProgress>();
        assert!(
            !progress.powered()
                && !progress.service_key
                && !progress.escaped
                && !progress.map_known
                && !progress.map_open
        );
    }
}
