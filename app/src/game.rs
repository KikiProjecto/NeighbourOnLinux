use crate::{
    Inspection, LaunchOptions,
    ai::{AlertState, Neighbor},
    audio::SoundCue,
    items::{Inventory, ItemKind, MysteryDoor, Pickup, PuzzleProgress},
    motion::{MotionHistory, record_movement},
    player::{MoveInput, Player},
    world::*,
};
use bevy::{
    ecs::system::SystemParam,
    input::mouse::AccumulatedMouseMotion,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow},
};

pub struct NeighborhoodPlugin;
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    Setup,
    Controls,
    Simulation,
    Interact,
    Presentation,
}
impl Plugin for NeighborhoodPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<HousePlan>()
            .init_resource::<CollisionWorld>()
            .init_resource::<Session>()
            .init_resource::<Inventory>()
            .init_resource::<PuzzleProgress>()
            .init_resource::<PlayerInput>()
            .add_message::<SoundCue>()
            .add_systems(Startup, setup.in_set(GameSet::Setup))
            .add_systems(
                PreUpdate,
                read_controls
                    .after(bevy::input::InputSystems)
                    .in_set(GameSet::Controls),
            )
            .add_systems(
                FixedUpdate,
                (
                    record_movement,
                    rebuild_collisions,
                    move_pickups,
                    move_player,
                    move_neighbor,
                )
                    .chain()
                    .in_set(GameSet::Simulation),
            )
            .add_systems(
                Update,
                (interact, ladder_controls, restart)
                    .chain()
                    .in_set(GameSet::Interact),
            )
            .add_systems(
                Update,
                (animate_mechanisms, sync_doors, sync_views)
                    .chain()
                    .after(GameSet::Interact)
                    .in_set(GameSet::Presentation),
            );
    }
}
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Outcome {
    #[default]
    Exploring,
    Caught,
    Escaped,
}
#[derive(Resource)]
pub struct Session {
    pub outcome: Outcome,
    pub paused: bool,
    pub flashlight: bool,
    pub attempts: u32,
    pub notice: String,
    pub notice_for: f32,
    pub prompt: String,
    pub footstep_in: f32,
    pub last_capture: Option<Vec3>,
}
impl Default for Session {
    fn default() -> Self {
        Self {
            outcome: Outcome::Exploring,
            paused: false,
            flashlight: true,
            attempts: 1,
            notice: "Something is behind the basement door. Don't let him hear you.".into(),
            notice_for: 8.,
            prompt: String::new(),
            footstep_in: 0.,
            last_capture: None,
        }
    }
}
impl Session {
    pub fn tell(&mut self, text: impl Into<String>) {
        self.notice = text.into();
        self.notice_for = 4.5;
    }
}
#[derive(Resource, Default)]
struct PlayerInput {
    forward: f32,
    strafe: f32,
    jump: bool,
    run: bool,
    crouch: bool,
}
#[derive(SystemParam)]
struct RunState<'w> {
    options: Res<'w, LaunchOptions>,
    session: ResMut<'w, Session>,
}
#[derive(SystemParam)]
struct RawInput<'w> {
    keys: Res<'w, ButtonInput<KeyCode>>,
    buttons: Res<'w, ButtonInput<MouseButton>>,
    motion: Res<'w, AccumulatedMouseMotion>,
}
#[derive(SystemParam)]
struct ProgressState<'w> {
    session: ResMut<'w, Session>,
    inventory: ResMut<'w, Inventory>,
    progress: ResMut<'w, PuzzleProgress>,
}
#[derive(SystemParam)]
struct StealthWorld<'w> {
    plan: Res<'w, HousePlan>,
    collision: Res<'w, CollisionWorld>,
    input: Res<'w, PlayerInput>,
}

#[derive(Component)]
pub struct GameCamera;
#[derive(Component)]
struct Flashlight;
#[derive(Component)]
struct Windmill {
    speed: f32,
}
#[derive(Component)]
pub(crate) struct Train {
    distance: f32,
    speed: f32,
}
#[derive(Component)]
enum DoorVisual {
    Basement,
    Locked(MysteryDoor),
    Board,
}
#[derive(Component)]
struct DoorMotion {
    rest: Transform,
    amount: f32,
    attempt: u32,
}
#[derive(Component)]
struct SwitchLamp {
    generator: bool,
}

#[derive(Resource)]
pub(crate) struct Art {
    cube: Handle<Mesh>,
    sphere: Handle<Mesh>,
    cylinder: Handle<Mesh>,
    cone: Handle<Mesh>,
    paints: Vec<Handle<StandardMaterial>>,
}
impl Art {
    #[cfg(test)]
    pub(crate) fn test_fixture() -> Self {
        Self {
            cube: default(),
            sphere: default(),
            cylinder: default(),
            cone: default(),
            paints: vec![Handle::default(); Paint::ALL.len()],
        }
    }
    pub(crate) fn material(&self, paint: Paint) -> Handle<StandardMaterial> {
        self.paints[paint as usize].clone()
    }
    fn mesh(&self, shape: Shape) -> Handle<Mesh> {
        match shape {
            Shape::Box => self.cube.clone(),
            Shape::Sphere => self.sphere.clone(),
            Shape::Cylinder => self.cylinder.clone(),
            Shape::Cone => self.cone.clone(),
        }
    }
    pub(crate) fn spawn(&self, commands: &mut Commands, piece: &Piece) -> Entity {
        commands
            .spawn((
                Name::new(piece.name),
                Mesh3d(self.mesh(piece.shape)),
                MeshMaterial3d(self.material(piece.paint)),
                piece.transform,
            ))
            .id()
    }
    fn prop(
        &self,
        commands: &mut Commands,
        name: &'static str,
        p: Vec3,
        size: Vec3,
        paint: Paint,
    ) -> Entity {
        self.spawn(
            commands,
            &Piece {
                name,
                transform: Transform::from_translation(p).with_scale(size),
                paint,
                shape: Shape::Box,
            },
        )
    }
    pub(crate) fn child(
        &self,
        commands: &mut Commands,
        parent: Entity,
        p: Vec3,
        size: Vec3,
        paint: Paint,
        shape: Shape,
    ) -> Entity {
        let id = self.spawn(
            commands,
            &Piece {
                name: "Procedural model part",
                transform: Transform::from_translation(p).with_scale(size),
                paint,
                shape,
            },
        );
        commands.entity(parent).add_child(id);
        id
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    plan: Res<HousePlan>,
    mut collision: ResMut<CollisionWorld>,
    options: Res<LaunchOptions>,
) {
    let art = Art {
        cube: meshes.add(Cuboid::default()),
        sphere: meshes.add(Sphere::new(1.).mesh().uv(12, 8)),
        cylinder: meshes.add(Cylinder::new(1., 1.).mesh().resolution(12)),
        cone: meshes.add(
            Cone {
                radius: 1.,
                height: 1.,
            }
            .mesh()
            .resolution(16),
        ),
        paints: Paint::ALL
            .into_iter()
            .map(|paint| materials.add(crate::materials::surface(paint, &mut images)))
            .collect(),
    };
    for (index, piece) in plan.pieces.iter().enumerate() {
        if !crate::props::spawn_piece(&mut commands, &art, piece, index) {
            let entity = art.spawn(&mut commands, piece);
            crate::effects::decorate(&mut commands, entity, piece, index);
            if index >= plan.underground_start {
                commands
                    .entity(entity)
                    .insert(crate::basement::UndergroundMesh {
                        center: piece.transform.translation,
                        radius: piece.transform.scale.length(),
                    });
            }
        }
    }
    rebuild_static_collisions(&plan, &PuzzleProgress::default(), &mut collision);
    commands.spawn((
        Name::new("Sun and moon key light"),
        crate::atmosphere::SkyLight,
        DirectionalLight {
            color: Color::srgb(0.56, 0.7, 0.95),
            illuminance: 2800.,
            shadow_maps_enabled: !options.low_graphics,
            ..default()
        },
        bevy::light::CascadeShadowConfigBuilder {
            num_cascades: if options.low_graphics { 1 } else { 2 },
            first_cascade_far_bound: if options.low_graphics { 16. } else { 20. },
            maximum_distance: if options.low_graphics { 40. } else { 75. },
            ..default()
        }
        .build(),
        Transform::from_xyz(-32., 44., 24.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
    // Small pools of amber in otherwise blue rooms. No wall-penetrating shadows are claimed.
    for level in 0..5 {
        let pos = Vec3::new(-5.5, FLOOR_Y + level as f32 * STOREY + 2.8, 2.);
        commands.spawn((
            PointLight {
                color: Color::srgb(1., 0.64, 0.28),
                intensity: 1100.,
                range: 12.,
                radius: 0.2,
                ..default()
            },
            Transform::from_translation(pos),
        ));
        art.prop(
            &mut commands,
            "Ceiling lamp",
            pos,
            Vec3::new(0.65, 0.14, 0.65),
            Paint::Glow,
        );
    }
    for pos in [
        Vec3::new(0., 2.65, 9.5),
        Vec3::new(-8., 2.7, -3.5),
        Vec3::new(14., 3., 14.),
    ] {
        commands.spawn((
            PointLight {
                color: Color::srgb(1., 0.52, 0.19),
                intensity: 650.,
                range: 9.,
                ..default()
            },
            Transform::from_translation(pos),
        ));
    }
    // The tower is above the regular floor lamps; give its control room a small,
    // warm practical light so the machinery remains readable at night.
    let tower_sconce = Vec3::new(4.18, 22.9, -6.);
    commands.spawn((
        Name::new("Tower gallery sconce"),
        PointLight {
            color: Color::srgb(1., 0.58, 0.28),
            intensity: 1500.,
            range: 9.,
            radius: 0.15,
            ..default()
        },
        Transform::from_translation(tower_sconce),
    ));
    art.prop(
        &mut commands,
        "Tower gallery lamp",
        tower_sconce + Vec3::X * 0.08,
        Vec3::new(0.16, 0.3, 0.24),
        Paint::Glow,
    );
    // An unlit moon and a few distant stars make the night sky authored and quiet.
    let moon = materials.add(StandardMaterial {
        base_color: Color::srgb(0.72, 0.79, 0.85),
        unlit: true,
        ..default()
    });
    commands.spawn((
        Mesh3d(art.sphere.clone()),
        MeshMaterial3d(moon.clone()),
        crate::atmosphere::CelestialBody::Moon,
        Transform::from_xyz(-27., 38., -45.).with_scale(Vec3::splat(2.4)),
    ));
    for i in 0..36 {
        let x = ((i * 73) % 151) as f32 - 75.;
        let z = -40. - ((i * 17) % 50) as f32;
        let y = 28. + ((i * 29) % 40) as f32;
        commands.spawn((
            Mesh3d(art.sphere.clone()),
            MeshMaterial3d(moon.clone()),
            crate::atmosphere::CelestialBody::Star,
            Transform::from_xyz(x, y, z).with_scale(Vec3::splat(0.055)),
        ));
    }
    let mut player = Player::new(PLAYER_SPAWN);
    player.yaw = 0.05;
    player.pitch = 0.25;
    if let Some(view) = options.inspect {
        let (eye, target) = inspection_view(view);
        player.pos = eye - Vec3::Y * 1.65;
        let direction = (target - eye).normalize();
        player.yaw = (-direction.x).atan2(-direction.z);
        player.pitch = direction.y.asin();
    }
    let history = MotionHistory::new(player.pos);
    commands.spawn((Name::new("Player controller"), player, history));
    let mut camera = commands.spawn((
        Name::new("First person camera"),
        Camera3d::default(),
        GameCamera,
        bevy::camera::Exposure::default(),
        bevy::camera::ShadowLodOrigin,
        Projection::Perspective(PerspectiveProjection {
            fov: if options.overview { 60_f32 } else { 75_f32 }.to_radians(),
            ..default()
        }),
        DistanceFog {
            color: Color::srgb(0.035, 0.055, 0.085),
            falloff: FogFalloff::Linear {
                start: 45.,
                end: 210.,
            },
            ..default()
        },
        Msaa::Off,
        bevy::anti_alias::fxaa::Fxaa::default(),
        crate::visuals::color_grade(),
        Transform::from_xyz(0., 2., 28.),
    ));
    if !options.low_graphics {
        camera.insert(bevy::post_process::bloom::Bloom {
            intensity: 0.06,
            ..bevy::post_process::bloom::Bloom::NATURAL
        });
    }
    if options.screenshot.is_some() || options.profile_encounter.is_some() {
        // Keep review screenshots at full size. The low-graphics encounter
        // replay uses its reduced target resolution so it measures the preset.
        let (width, height) = if options.low_graphics && options.screenshot.is_none() {
            (960, 540)
        } else {
            (1280, 720)
        };
        let target = images.add(Image::new_target_texture(
            width,
            height,
            bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
            None,
        ));
        camera.insert(bevy::camera::RenderTarget::Image(target.clone().into()));
        commands.insert_resource(crate::capture::CaptureTarget(target));
    }
    commands.spawn((
        Name::new("Player flashlight"),
        Flashlight,
        SpotLight {
            intensity: 48_000.,
            range: 26.,
            color: Color::srgb(1., 0.86, 0.65),
            inner_angle: 0.18,
            outer_angle: 0.38,
            shadow_maps_enabled: !options.low_graphics,
            ..default()
        },
        Transform::default(),
    ));
    crate::character::spawn_neighbor(&mut commands, &art);
    spawn_mechanisms(&mut commands, &art);
    for (kind, pos) in [
        (ItemKind::Crowbar, Vec3::new(-8., 4.85, -6.9)),
        (ItemKind::BlueKey, Vec3::new(-4.85, 1.1, 5.55)),
        (ItemKind::GreenKey, Vec3::new(-8., 8.3, -7.4)),
        (ItemKind::RedKey, Vec3::new(-10.4, 15.4, -8.6)),
        (ItemKind::Umbrella, Vec3::new(-4., 7.75, 3.)),
        (ItemKind::Box, Vec3::new(2.5, 0.38, 12.)),
        (ItemKind::Box, Vec3::new(-2.5, 0.38, 13.)),
        (ItemKind::Box, Vec3::new(3.7, 3.98, -1.)),
    ] {
        spawn_pickup(&mut commands, &art, kind, pos);
    }
    commands.insert_resource(art);
}
fn spawn_pickup(commands: &mut Commands, art: &Art, kind: ItemKind, pos: Vec3) {
    let root = commands
        .spawn((
            Name::new(kind.label()),
            Transform::from_translation(pos),
            Visibility::Inherited,
            Pickup {
                kind,
                home: pos,
                velocity: Vec3::ZERO,
                held: false,
            },
        ))
        .id();
    match kind {
        ItemKind::Box => {
            art.child(
                commands,
                root,
                Vec3::ZERO,
                Vec3::splat(0.64),
                Paint::Wood,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::Y * 0.325,
                Vec3::new(0.17, 0.012, 0.64),
                Paint::Cream,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::Z * 0.325,
                Vec3::new(0.17, 0.64, 0.012),
                Paint::Cream,
                Shape::Box,
            );
        }
        ItemKind::Crowbar => {
            art.child(
                commands,
                root,
                Vec3::ZERO,
                Vec3::new(0.075, 0.075, 0.85),
                Paint::Red,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::new(0.09, 0., -0.4),
                Vec3::new(0.25, 0.08, 0.08),
                Paint::Metal,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::new(0.2, 0.03, -0.4),
                Vec3::new(0.06, 0.11, 0.08),
                Paint::Metal,
                Shape::Box,
            );
        }
        ItemKind::Umbrella => {
            art.child(
                commands,
                root,
                Vec3::ZERO,
                Vec3::new(0.04, 0.8, 0.04),
                Paint::Metal,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::Y * 0.13,
                Vec3::new(0.15, 0.34, 0.15),
                Paint::Yellow,
                Shape::Sphere,
            );
            art.child(
                commands,
                root,
                Vec3::new(0.07, -0.4, 0.),
                Vec3::new(0.18, 0.06, 0.07),
                Paint::DarkWood,
                Shape::Box,
            );
        }
        ItemKind::BlueKey | ItemKind::GreenKey | ItemKind::RedKey => {
            let paint = match kind {
                ItemKind::BlueKey => Paint::PaleBlue,
                ItemKind::GreenKey => Paint::Leaf,
                _ => Paint::Red,
            };
            art.child(
                commands,
                root,
                Vec3::ZERO,
                Vec3::new(0.065, 0.07, 0.42),
                paint,
                Shape::Box,
            );
            art.child(
                commands,
                root,
                Vec3::new(0., 0., -0.27),
                Vec3::new(0.14, 0.035, 0.14),
                paint,
                Shape::Cylinder,
            );
            art.child(
                commands,
                root,
                Vec3::new(0., 0.04, -0.27),
                Vec3::new(0.065, 0.006, 0.065),
                Paint::DarkWood,
                Shape::Cylinder,
            );
            for z in [0.1, 0.2] {
                art.child(
                    commands,
                    root,
                    Vec3::new(0.065, 0., z),
                    Vec3::new(0.13, 0.07, 0.055),
                    paint,
                    Shape::Box,
                );
            }
        }
    }
}
fn spawn_mechanisms(commands: &mut Commands, art: &Art) {
    let windmill = commands
        .spawn((
            Name::new("Windmill rotor"),
            Windmill { speed: 0.28 },
            Transform::from_xyz(-3.7, 28., -3.5),
            Visibility::default(),
        ))
        .id();
    art.child(
        commands,
        windmill,
        Vec3::ZERO,
        Vec3::splat(0.4),
        Paint::Metal,
        Shape::Sphere,
    );
    for i in 0..4 {
        let angle = i as f32 * std::f32::consts::FRAC_PI_2;
        let dir = Quat::from_rotation_z(angle) * Vec3::Y;
        let id = art.child(
            commands,
            windmill,
            dir * 2.5,
            Vec3::new(0.2, 5., 0.16),
            Paint::DarkWood,
            Shape::Box,
        );
        commands.entity(id).insert(
            Transform::from_translation(dir * 2.5)
                .with_rotation(Quat::from_rotation_z(angle))
                .with_scale(Vec3::new(0.2, 5., 0.16)),
        );
        let sail = art.child(
            commands,
            windmill,
            dir * 3.4,
            Vec3::new(1.1, 2.7, 0.07),
            Paint::Cream,
            Shape::Box,
        );
        commands.entity(sail).insert(
            Transform::from_translation(dir * 3.4)
                .with_rotation(Quat::from_rotation_z(angle + 0.12))
                .with_scale(Vec3::new(1.1, 2.7, 0.07)),
        );
        for j in 0..4 {
            let rib = art.child(
                commands,
                windmill,
                dir * (2.3 + j as f32 * 0.7),
                Vec3::new(1.25, 0.065, 0.1),
                Paint::DarkWood,
                Shape::Box,
            );
            commands.entity(rib).insert(
                Transform::from_translation(dir * (2.3 + j as f32 * 0.7))
                    .with_rotation(Quat::from_rotation_z(angle))
                    .with_scale(Vec3::new(1.25, 0.065, 0.1)),
            );
        }
    }
    let train = commands
        .spawn((
            Name::new("Elevated trolley"),
            Train {
                distance: 2.,
                speed: 0.,
            },
            Transform::from_xyz(-14.5, 11.7, 11.6),
            Visibility::default(),
        ))
        .id();
    art.child(
        commands,
        train,
        Vec3::new(0., 0.3, 0.),
        Vec3::new(2.7, 0.6, 1.8),
        Paint::Red,
        Shape::Box,
    );
    art.child(
        commands,
        train,
        Vec3::new(0., 1.9, 0.),
        Vec3::new(2.9, 0.18, 2.),
        Paint::Roof,
        Shape::Box,
    );
    for x in [-1.1, 1.1] {
        for z in [-0.75, 0.75] {
            art.child(
                commands,
                train,
                Vec3::new(x, 1.1, z),
                Vec3::new(0.1, 1.8, 0.1),
                Paint::White,
                Shape::Box,
            );
        }
    }
    for door in MysteryDoor::ALL {
        spawn_hinged_door(
            commands,
            art,
            door.label(),
            door.center(),
            match door {
                MysteryDoor::Archive => Paint::Blue,
                MysteryDoor::Greenhouse => Paint::Teal,
                MysteryDoor::Tower => Paint::Red,
            },
            DoorVisual::Locked(door),
        );
    }
    spawn_hinged_door(
        commands,
        art,
        "Basement electronic door",
        BASEMENT_DOOR,
        Paint::Black,
        DoorVisual::Basement,
    );
    for i in 0..3 {
        let id = art.prop(
            commands,
            "Basement boards",
            BASEMENT_DOOR + Vec3::new(0., -0.7 + i as f32 * 0.7, 0.22),
            Vec3::new(2.65, 0.26, 0.14),
            Paint::Wood,
        );
        commands.entity(id).insert((
            DoorVisual::Board,
            DoorMotion {
                rest: Transform::from_translation(
                    BASEMENT_DOOR + Vec3::new(0., -0.7 + i as f32 * 0.7, 0.22),
                )
                .with_scale(Vec3::new(2.65, 0.26, 0.14)),
                amount: 0.,
                attempt: 1,
            },
        ));
    }
    for (pos, generator) in [(GENERATOR, true), (WINDMILL_SWITCH, false)] {
        let id = art.prop(
            commands,
            "Power switch indicator",
            pos + Vec3::Z * 0.45,
            Vec3::splat(0.18),
            Paint::Red,
        );
        commands.entity(id).insert(SwitchLamp { generator });
    }
}
fn spawn_hinged_door(
    commands: &mut Commands,
    art: &Art,
    name: &'static str,
    center: Vec3,
    paint: Paint,
    kind: DoorVisual,
) {
    let rest = Transform::from_translation(center - Vec3::X * 1.125);
    let root = commands
        .spawn((
            Name::new(name),
            rest,
            Visibility::Inherited,
            kind,
            DoorMotion {
                rest,
                amount: 0.,
                attempt: 1,
            },
        ))
        .id();
    art.child(
        commands,
        root,
        Vec3::X * 1.125,
        Vec3::new(2.25, 2.6, 0.22),
        paint,
        Shape::Box,
    );
    for y in [-0.66, 0.66] {
        art.child(
            commands,
            root,
            Vec3::new(1.125, y, 0.12),
            Vec3::new(1.85, 0.93, 0.04),
            Paint::DarkWood,
            Shape::Box,
        );
    }
    art.child(
        commands,
        root,
        Vec3::new(1.92, -0.05, 0.2),
        Vec3::new(0.08, 0.22, 0.12),
        Paint::Metal,
        Shape::Box,
    );
    art.child(
        commands,
        root,
        Vec3::new(1.81, -0.05, 0.28),
        Vec3::new(0.3, 0.07, 0.07),
        Paint::Yellow,
        Shape::Box,
    );
    for y in [-0.9, 0.9] {
        art.child(
            commands,
            root,
            Vec3::new(0.045, y, 0.14),
            Vec3::new(0.08, 0.2, 0.08),
            Paint::Metal,
            Shape::Box,
        );
    }
}
fn basement_collider() -> Aabb {
    Aabb::from_center(BASEMENT_DOOR, Vec3::new(2.25, 2.6, 0.3))
}

fn read_controls(
    raw: RawInput,
    run: RunState,
    mut input: ResMut<PlayerInput>,
    mut player: Query<&mut Player>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
    mut exit: MessageWriter<AppExit>,
) {
    let RawInput {
        keys,
        buttons,
        motion,
    } = raw;
    let RunState {
        options,
        mut session,
    } = run;
    if keys.just_pressed(KeyCode::F10) {
        exit.write(AppExit::Success);
    }
    if keys.just_pressed(KeyCode::Escape) {
        session.paused = !session.paused;
    }
    if buttons.just_pressed(MouseButton::Left) {
        session.paused = false;
    }
    if let Ok((window, mut cursor)) = windows.single_mut() {
        let grabbing = window.focused
            && !session.paused
            && options.screenshot.is_none()
            && options.frames.is_none()
            && options.profile_encounter.is_none();
        cursor.grab_mode = if grabbing {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
        cursor.visible = !grabbing;
        if !window.focused
            && options.screenshot.is_none()
            && options.frames.is_none()
            && options.profile_encounter.is_none()
        {
            session.paused = true;
        }
    }
    if session.paused || session.outcome != Outcome::Exploring || options.overview {
        *input = PlayerInput::default();
        return;
    }
    input.forward = f32::from(keys.pressed(KeyCode::KeyW)) - f32::from(keys.pressed(KeyCode::KeyS));
    input.strafe = f32::from(keys.pressed(KeyCode::KeyD)) - f32::from(keys.pressed(KeyCode::KeyA));
    input.jump |= keys.just_pressed(KeyCode::Space);
    input.run = keys.pressed(KeyCode::ShiftLeft);
    input.crouch = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::KeyC);
    if let Ok(mut player) = player.single_mut() {
        player.yaw -= motion.delta.x * 0.0023;
        player.pitch = (player.pitch - motion.delta.y * 0.0023).clamp(-1.48, 1.48);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        session.flashlight = !session.flashlight;
    }
}
fn move_player(
    time: Res<Time<Fixed>>,
    collision: Res<CollisionWorld>,
    run: RunState,
    inventory: Res<Inventory>,
    mut input: ResMut<PlayerInput>,
    mut players: Query<&mut Player>,
    mut sound: MessageWriter<SoundCue>,
) {
    let RunState {
        options,
        mut session,
    } = run;
    if session.paused || session.outcome != Outcome::Exploring || options.overview {
        return;
    }
    let Ok(mut p) = players.single_mut() else {
        return;
    };
    let dt = time.delta_secs();
    let fall_speed = p.vel.y;
    p.step(
        dt,
        &MoveInput {
            forward: input.forward,
            strafe: input.strafe,
            jump: input.jump,
            run: input.run,
            crouch: input.crouch,
            look: Vec2::ZERO,
        },
        &collision.solids,
    );
    if p.on_ground && fall_speed < -6. {
        sound.write(SoundCue::Impact(p.pos));
    }
    input.jump = false;
    if inventory.umbrella && input.run && p.vel.y < -3.0 {
        p.vel.y = -3.0;
    }
    if p.pos.y < -12. {
        p.pos = PLAYER_SPAWN;
        p.vel = Vec3::ZERO;
    }
    session.footstep_in -= dt;
    if p.on_ground && p.vel.xz().length() > 1. && session.footstep_in <= 0. {
        sound.write(SoundCue::Step);
        session.footstep_in = if input.run { 0.3 } else { 0.5 };
    }
}
fn move_neighbor(
    time: Res<Time<Fixed>>,
    run: RunState,
    world: StealthWorld,
    players: Query<&Player>,
    mut neighbors: Query<&mut Neighbor>,
    mut sound: MessageWriter<SoundCue>,
    mut noise_in: Local<f32>,
) {
    let RunState {
        options,
        mut session,
    } = run;
    let StealthWorld {
        plan,
        collision,
        input,
    } = world;
    if session.paused || session.outcome != Outcome::Exploring || options.overview {
        return;
    }
    let (Ok(p), Ok(mut n)) = (players.single(), neighbors.single_mut()) else {
        return;
    };
    let dt = time.delta_secs();
    *noise_in -= dt;
    if input.run && p.vel.xz().length() > 2. && p.on_ground && *noise_in <= 0. {
        n.hear(p.pos, 14., &collision.solids);
        *noise_in = 0.65;
    }
    let before = n.state;
    if n.tick(dt, p, &plan, &collision.solids) {
        session.outcome = Outcome::Caught;
        session.last_capture = Some(p.pos);
        sound.write(SoundCue::Caught);
    } else if n.state == AlertState::Chase && before != AlertState::Chase {
        sound.write(SoundCue::Alert);
    }
}

/// A target must be close, within the reticle cone, and unobstructed.
pub(crate) fn can_interact(p: &Player, point: Vec3, collision: &[Aabb]) -> bool {
    let delta = point - p.eye();
    let distance = delta.length();
    if distance > 2.8 || distance < 0.01 || p.forward().dot(delta / distance) < 0.79 {
        return false;
    }
    // End just in front of the target so the target's own housing does not occlude it.
    line_of_sight(p.eye(), point - delta.normalize() * 0.28, collision)
}
fn interact(
    keys: Res<ButtonInput<KeyCode>>,
    players: Query<&Player>,
    collision: Res<CollisionWorld>,
    state: ProgressState,
    mut pickups: Query<(Entity, &mut Pickup, &mut Transform, &mut Visibility)>,
    mut neighbors: Query<&mut Neighbor>,
    mut sound: MessageWriter<SoundCue>,
) {
    let ProgressState {
        mut session,
        mut inventory,
        mut progress,
    } = state;
    session.prompt.clear();
    if session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    let Ok(p) = players.single() else {
        return;
    };
    if inventory.carrying.is_some()
        && (keys.just_pressed(KeyCode::KeyQ) || keys.just_pressed(KeyCode::KeyG))
    {
        let throwing = keys.just_pressed(KeyCode::KeyG);
        if let Some(held) = inventory.carrying.take()
            && let Ok((_, mut item, mut tf, _)) = pickups.get_mut(held)
        {
            item.held = false;
            tf.translation = p.eye() + p.forward() * 0.9;
            item.velocity = p.forward() * if throwing { 12. } else { 1.8 }
                + Vec3::Y * if throwing { 2. } else { 0. };
            sound.write(SoundCue::Throw);
        }
        return;
    }
    for (entity, mut item, tf, mut visibility) in &mut pickups {
        if item.held || *visibility == Visibility::Hidden {
            continue;
        }
        if can_interact(
            p,
            tf.translation,
            &collision.solids[..collision.static_count],
        ) {
            session.prompt = format!("[E] Pick up {}", item.kind.label());
            if keys.just_pressed(KeyCode::KeyE) {
                match item.kind {
                    ItemKind::BlueKey | ItemKind::GreenKey | ItemKind::RedKey => {
                        match item.kind {
                            ItemKind::BlueKey => inventory.blue_key = true,
                            ItemKind::GreenKey => inventory.green_key = true,
                            _ => inventory.red_key = true,
                        }
                        *visibility = Visibility::Hidden;
                        session.tell(match item.kind {
                            ItemKind::BlueKey => "Blue key: the archive door on floor 3.",
                            ItemKind::GreenKey => "Green key: the greenhouse on floor 5.",
                            _ => "Red key: the tower in the roof garden. The machinery is at the top.",
                        });
                    }
                    ItemKind::Crowbar => {
                        inventory.crowbar = true;
                        *visibility = Visibility::Hidden;
                        session.tell(
                            "Crowbar collected. Find the windmill control at the top of the tower.",
                        );
                    }
                    ItemKind::Umbrella => {
                        inventory.umbrella = true;
                        *visibility = Visibility::Hidden;
                        session.tell(
                            "Umbrella collected. Hold Shift while falling to slow your descent.",
                        );
                    }
                    ItemKind::Box => {
                        if inventory.carrying.is_none() {
                            inventory.carrying = Some(entity);
                            item.held = true;
                        } else {
                            session.tell("Your hands are full. Q to drop, G to throw.");
                        }
                    }
                }
                sound.write(SoundCue::Pickup);
            }
            return;
        }
    }
    for door in MysteryDoor::ALL {
        if door.is_open(&progress) {
            continue;
        }
        if can_interact(p, door.center() + Vec3::Z * 0.5, &collision.solids) {
            session.prompt = format!("[E] Open {}", door.label());
            if keys.just_pressed(KeyCode::KeyE) {
                match door.unlock(&mut progress, &inventory) {
                    Ok(message) => {
                        session.tell(message);
                        sound.write(SoundCue::Door);
                    }
                    Err(message) => session.tell(message),
                }
            }
            return;
        }
    }
    let target = if can_interact(p, WINDMILL_SWITCH + Vec3::Z * 0.5, &collision.solids) {
        Some((
            0,
            if progress.windmill_stopped {
                "The windmill has stopped"
            } else {
                "[E] Stop the windmill"
            },
        ))
    } else if can_interact(p, GENERATOR + Vec3::Z * 0.85, &collision.solids) {
        Some((
            1,
            if progress.power_off {
                "The generator is off"
            } else {
                "[E] Switch off the generator"
            },
        ))
    } else if can_interact(p, FREEZER + Vec3::Z * 0.9, &collision.solids) {
        Some((
            2,
            if inventory.keycard {
                "Keycard already collected"
            } else {
                "[E] Open the freezer"
            },
        ))
    } else if !progress.basement_open
        && can_interact(p, BASEMENT_DOOR + Vec3::Z * 0.45, &collision.solids)
    {
        Some((
            3,
            if progress.boards_removed {
                "[E] Use the keycard"
            } else {
                "[E] Remove the basement boards"
            },
        ))
    } else if can_interact(p, TRAIN_SWITCH + Vec3::Z * 0.7, &collision.solids) {
        Some((4, "[E] Start / stop the trolley"))
    } else {
        None
    };
    if let Some((id, prompt)) = target {
        session.prompt = prompt.into();
        if keys.just_pressed(KeyCode::KeyE) {
            let result = match id {
                0 => {
                    progress.windmill_stopped = true;
                    Ok("The windmill has stopped. You can now switch off the red generator.")
                }
                1 => progress
                    .disable_generator()
                    .map(|()| "Power is off. The keycard is in the kitchen freezer."),
                2 => progress
                    .take_keycard(&mut inventory)
                    .map(|()| "Keycard collected. Return to the basement door."),
                3 => progress.open_basement(&inventory).map(|open| {
                    if open {
                        "The basement is open. Descend the stairs and read the maintenance map."
                    } else {
                        "Boards removed. The electronic lock is still active."
                    }
                }),
                _ => {
                    progress.train_running = !progress.train_running;
                    Ok(if progress.train_running {
                        "The trolley is starting."
                    } else {
                        "The trolley is stopping."
                    })
                }
            };
            match result {
                Ok(message) => {
                    session.tell(message);
                    sound.write(SoundCue::Door);
                }
                Err(message) => session.tell(message),
            }
            if let Ok(mut n) = neighbors.single_mut() {
                n.hear(p.pos, 12., &collision.solids);
            }
        }
    }
}
fn move_pickups(
    time: Res<Time<Fixed>>,
    players: Query<&Player>,
    collision: Res<CollisionWorld>,
    mut pickups: Query<(Entity, &mut Pickup, &mut Transform, &Visibility)>,
    mut neighbors: Query<&mut Neighbor>,
    session: Res<Session>,
    mut sound: MessageWriter<SoundCue>,
) {
    if session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    let Ok(p) = players.single() else {
        return;
    };
    let dt = time.delta_secs();
    for (entity, mut item, mut tf, visibility) in &mut pickups {
        if item.kind != ItemKind::Box || *visibility == Visibility::Hidden {
            continue;
        }
        if item.held {
            let right = Vec3::new(p.yaw.cos(), 0., -p.yaw.sin());
            let target = p.eye() + p.forward() * 0.85 + right * 0.3 - Vec3::Y * 0.4;
            tf.translation = tf.translation.lerp(target, 1. - (-18. * dt).exp());
            tf.rotation = tf
                .rotation
                .slerp(Quat::from_rotation_y(p.yaw), 1. - (-16. * dt).exp());
            continue;
        }
        if item.velocity.length_squared() < 0.001 {
            let support = Aabb::from_center(tf.translation - Vec3::Y * 0.02, Vec3::splat(0.64));
            if collision.solids[..collision.static_count]
                .iter()
                .any(|c| support.overlaps(c))
                || collision
                    .loose_boxes
                    .iter()
                    .any(|(other, c)| *other != entity && support.overlaps(c))
            {
                continue;
            }
        }
        item.velocity.y -= 20. * dt;
        let steps = ((item.velocity.length() * dt / 0.12).ceil() as u32).max(1);
        for _ in 0..steps {
            let start = tf.translation;
            let target = start + item.velocity * dt / steps as f32;
            let bounds = Aabb::from_center(target, Vec3::splat(0.64));
            if collision.solids[..collision.static_count]
                .iter()
                .any(|c| bounds.overlaps(c))
                || collision
                    .loose_boxes
                    .iter()
                    .any(|(other, c)| *other != entity && bounds.overlaps(c))
            {
                let loudness = item.velocity.length().min(22.);
                item.velocity = Vec3::ZERO;
                if loudness > 3. {
                    if let Ok(mut n) = neighbors.single_mut() {
                        n.hear(start, loudness, &collision.solids);
                    }
                    sound.write(SoundCue::Impact(start));
                }
                break;
            }
            tf.translation = target;
        }
        if tf.translation.y < -10. {
            tf.translation = item.home;
            item.velocity = Vec3::ZERO;
        }
    }
}
fn restart(
    keys: Res<ButtonInput<KeyCode>>,
    state: ProgressState,
    plan: Res<HousePlan>,
    mut collision: ResMut<CollisionWorld>,
    mut players: Query<&mut Player>,
    mut neighbors: Query<&mut Neighbor>,
    mut pickups: Query<(&mut Pickup, &mut Transform, &mut Visibility)>,
) {
    let ProgressState {
        mut session,
        mut inventory,
        mut progress,
    } = state;
    if !keys.just_pressed(KeyCode::KeyR) || session.outcome == Outcome::Exploring {
        return;
    }
    let memory = session.last_capture;
    let attempts = session.attempts + 1;
    *session = Session {
        attempts,
        last_capture: memory,
        ..default()
    };
    *inventory = Inventory::default();
    *progress = PuzzleProgress::default();
    rebuild_static_collisions(&plan, &progress, &mut collision);
    if let Ok(mut p) = players.single_mut() {
        *p = Player::new(PLAYER_SPAWN);
        p.pitch = 0.25;
    }
    if let Ok(mut n) = neighbors.single_mut() {
        n.reset(memory);
    }
    for (mut item, mut tf, mut visibility) in &mut pickups {
        item.held = false;
        item.velocity = Vec3::ZERO;
        tf.translation = item.home;
        tf.rotation = Quat::IDENTITY;
        *visibility = Visibility::Inherited;
    }
    if memory.is_some() {
        session.tell("He remembers where you were caught. Try a different route.");
    }
}
fn animate_mechanisms(
    time: Res<Time>,
    progress: Res<PuzzleProgress>,
    session: Res<Session>,
    mut attempt: Local<u32>,
    mut rotors: Query<(&mut Windmill, &mut Transform)>,
    mut trains: Query<(&mut Train, &mut Transform), Without<Windmill>>,
) {
    if session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    let dt = time.delta_secs();
    let reset = *attempt != session.attempts;
    *attempt = session.attempts;
    for (mut rotor, mut tf) in &mut rotors {
        if reset {
            rotor.speed = 0.28;
            tf.rotation = Quat::IDENTITY;
        }
        let target = if progress.windmill_stopped { 0. } else { 0.28 };
        rotor.speed += (target - rotor.speed) * (1. - (-3. * dt).exp());
        tf.rotate_z(-dt * rotor.speed);
    }
    for (mut train, mut tf) in &mut trains {
        if reset {
            train.distance = 2.;
            train.speed = 0.;
            tf.rotation = Quat::from_rotation_y(std::f32::consts::FRAC_PI_2);
        }
        let target = if progress.train_running { 4. } else { 0. };
        train.speed += (target - train.speed) * (1. - (-2.5 * dt).exp());
        train.distance = (train.distance + dt * train.speed).rem_euclid(113.2);
        let (position, yaw) = train_pose(train.distance);
        tf.translation = position;
        tf.rotation = tf
            .rotation
            .slerp(Quat::from_rotation_y(yaw), 1. - (-8. * dt).exp());
    }
}
fn train_pose(distance: f32) -> (Vec3, f32) {
    let d = distance.rem_euclid(113.2);
    let (x, z, yaw) = if d < 33. {
        (-16.5 + d, 11.6, std::f32::consts::FRAC_PI_2)
    } else if d < 56.6 {
        (16.5, 11.6 - (d - 33.), std::f32::consts::PI)
    } else if d < 89.6 {
        (16.5 - (d - 56.6), -12., -std::f32::consts::FRAC_PI_2)
    } else {
        (-16.5, -12. + (d - 89.6), 0.)
    };
    (Vec3::new(x, 11.7, z), yaw)
}
fn sync_doors(
    time: Res<Time>,
    session: Res<Session>,
    progress: Res<PuzzleProgress>,
    art: Res<Art>,
    mut doors: Query<(
        &DoorVisual,
        &mut DoorMotion,
        &mut Transform,
        &mut Visibility,
    )>,
    mut lamps: Query<(&SwitchLamp, &mut MeshMaterial3d<StandardMaterial>)>,
) {
    for (door, mut motion, mut tf, mut visible) in &mut doors {
        if motion.attempt != session.attempts {
            motion.amount = 0.;
            motion.attempt = session.attempts;
        }
        let open = match door {
            DoorVisual::Basement => progress.basement_open,
            DoorVisual::Board => progress.boards_removed,
            DoorVisual::Locked(kind) => kind.is_open(&progress),
        };
        if !session.paused {
            motion.amount +=
                (f32::from(open) - motion.amount) * (1. - (-9. * time.delta_secs()).exp());
        }
        *tf = motion.rest;
        *visible = Visibility::Inherited;
        if matches!(door, DoorVisual::Board) {
            tf.translation.y -= motion.amount * 1.6;
            tf.translation.z += motion.amount * 0.35;
            tf.rotate_z(motion.amount * 0.35);
            if motion.amount > 0.99 {
                *visible = Visibility::Hidden;
            }
        } else {
            tf.rotate_y(-motion.amount * 1.5);
        }
    }
    for (lamp, mut material) in &mut lamps {
        let off = if lamp.generator {
            progress.power_off
        } else {
            progress.windmill_stopped
        };
        material.0 = art.material(if off { Paint::Glow } else { Paint::Red });
    }
}

type FlashlightView = (With<Flashlight>, Without<GameCamera>);
fn sync_views(
    time: Res<Time>,
    fixed_time: Res<Time<Fixed>>,
    run: RunState,
    players: Query<(&Player, Option<&MotionHistory>)>,
    mut cameras: Query<&mut Transform, With<GameCamera>>,
    mut flashlights: Query<(&mut Transform, &mut SpotLight), FlashlightView>,
) {
    let RunState {
        options,
        mut session,
    } = run;
    if !session.paused {
        session.notice_for = (session.notice_for - time.delta_secs()).max(0.);
    }
    let Ok((p, history)) = players.single() else {
        return;
    };
    let rendered_pos = if session.paused || session.outcome != Outcome::Exploring {
        p.pos
    } else {
        history.map_or(p.pos, |h| h.position(p.pos, fixed_time.overstep_fraction()))
    };
    let eye = rendered_pos + Vec3::Y * (p.eye().y - p.pos.y);
    for mut camera in &mut cameras {
        let transform = if let Some(view) = options.inspect {
            let (eye, target) = inspection_view(view);
            Transform::from_translation(eye).looking_at(target, Vec3::Y)
        } else if options.overview {
            Transform::from_xyz(-29., 16., 39.).looking_at(Vec3::new(-1., 14., -1.), Vec3::Y)
        } else {
            Transform::from_translation(eye).looking_to(p.forward(), Vec3::Y)
        };
        camera.set_if_neq(transform);
    }
    for (mut tf, mut light) in &mut flashlights {
        tf.set_if_neq(
            Transform::from_translation(eye - Vec3::Y * 0.12).looking_to(p.forward(), Vec3::Y),
        );
        let intensity = if session.flashlight && (!options.overview || options.inspect.is_some()) {
            48_000.
        } else {
            0.
        };
        if light.intensity != intensity {
            light.intensity = intensity;
        }
    }
}

fn rebuild_collisions(
    plan: Res<HousePlan>,
    progress: Res<PuzzleProgress>,
    mut collision: ResMut<CollisionWorld>,
    pickups: Query<(Entity, &Pickup, &Transform, &Visibility)>,
) {
    if progress.is_changed() || collision.static_count == 0 {
        rebuild_static_collisions(&plan, &progress, &mut collision);
    } else {
        // The authored geometry and keyed doors are stable across ordinary
        // fixed ticks. Keep that snapshot and replace only loose-box entries.
        let static_count = collision.static_count;
        collision.solids.truncate(static_count);
        collision.loose_boxes.clear();
    }
    for (entity, item, tf, visibility) in &pickups {
        if item.kind == ItemKind::Box && !item.held && *visibility != Visibility::Hidden {
            let bounds = Aabb::from_center(tf.translation, Vec3::splat(0.64));
            collision.loose_boxes.push((entity, bounds));
            collision.solids.push(bounds);
        }
    }
}
fn rebuild_static_collisions(
    plan: &HousePlan,
    progress: &PuzzleProgress,
    collision: &mut CollisionWorld,
) {
    collision.solids.clear();
    collision.solids.extend_from_slice(&plan.colliders);
    if !progress.basement_open {
        collision.solids.push(basement_collider());
    }
    for door in MysteryDoor::ALL {
        if !door.is_open(progress) {
            collision
                .solids
                .push(Aabb::from_center(door.center(), Vec3::new(2.25, 2.6, 0.22)));
        }
    }
    collision.static_count = collision.solids.len();
    collision.loose_boxes.clear();
}
fn ladder_controls(
    keys: Res<ButtonInput<KeyCode>>,
    progress: Res<PuzzleProgress>,
    collision: Res<CollisionWorld>,
    mut players: Query<&mut Player>,
    mut session: ResMut<Session>,
    mut sound: MessageWriter<SoundCue>,
) {
    if !progress.tower_open || session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    let Ok(mut p) = players.single_mut() else {
        return;
    };
    let upper = p.pos.y > 20.;
    let target = Vec3::new(0.85, if upper { 22.5 } else { 19.2 }, -6.5);
    if can_interact(&p, target, &collision.solids) {
        session.prompt = if upper {
            "[E] Climb down the tower ladder"
        } else {
            "[E] Climb up to the windmill machinery"
        }
        .into();
        if keys.just_pressed(KeyCode::KeyE) {
            p.pos = Vec3::new(1.8, if upper { 18.12 } else { 21.72 }, -6.2);
            p.vel = Vec3::ZERO;
            session.tell(if upper {
                "Back in the roof garden."
            } else {
                "Listen to the dynamo. The windmill supplies power to the freezer below."
            });
            sound.write(SoundCue::Door);
        }
    }
}

fn inspection_view(view: Inspection) -> (Vec3, Vec3) {
    match view {
        Inspection::Basement => {
            let p = crate::basement::ROOMS[0].center();
            (
                p + Vec3::new(2.4, 1.8, 2.8),
                p + Vec3::new(-0.3, 1.15, -3.2),
            )
        }
        Inspection::BasementLab => {
            let p = crate::basement::ROOMS[9].center();
            (p + Vec3::new(2.9, 1.8, 2.8), p + Vec3::new(-0.5, 1.1, -1.5))
        }
        Inspection::BasementVault => {
            let p = crate::basement::ROOMS[10].center();
            (p + Vec3::new(2.9, 1.8, 2.8), p + Vec3::new(0., 1.15, 0.))
        }
        Inspection::Workshop => (Vec3::new(-4.5, 5.3, -3.7), Vec3::new(-8.5, 4.9, -7.)),
        Inspection::Archive => (Vec3::new(-7.2, 8.9, -5.5), Vec3::new(-9., 8.5, -7.6)),
        Inspection::Station => (Vec3::new(-5., 12.55, 5.8), TRAIN_SWITCH),
        // Look back through the upper gallery at its listening horns and dial.
        Inspection::Tower => (Vec3::new(1.95, 23.0, -6.65), Vec3::new(3.2, 22.9, -7.72)),
        Inspection::Kitchen => (Vec3::new(-3.6, 2.2, -5.5), Vec3::new(-5.8, 1.4, -9.)),
        Inspection::LivingRoom => (Vec3::new(-4., 2.0, 2.7), Vec3::new(-8., 1.4, 0.8)),
        Inspection::Bathroom => (Vec3::new(1., 5.65, -5.7), Vec3::new(4.1, 5.3, -8.)),
        Inspection::Gallery => (Vec3::new(-1.3, 9.05, -5.6), Vec3::new(-4.5, 8.6, -7.5)),
        Inspection::Neighbor => (
            NEIGHBOR_SPAWN + Vec3::new(2.1, 1.5, 3.6),
            NEIGHBOR_SPAWN + Vec3::Y * 1.25,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn retry_rebuilds_a_consistent_collision_snapshot_before_another_render_frame() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<HousePlan>()
            .init_resource::<CollisionWorld>()
            .init_resource::<Session>()
            .init_resource::<Inventory>()
            .init_resource::<PuzzleProgress>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<SoundCue>()
            .add_systems(Update, (interact, restart).chain());
        app.world_mut().spawn(Player::new(PLAYER_SPAWN));
        app.world_mut().spawn(Neighbor::default());
        app.world_mut().spawn((
            Pickup {
                kind: ItemKind::BlueKey,
                home: Vec3::new(-4.85, 1.1, 5.55),
                velocity: Vec3::ZERO,
                held: false,
            },
            Transform::from_xyz(-4.85, 1.1, 5.55),
            Visibility::Inherited,
        ));
        let plan = HousePlan::default();
        rebuild_static_collisions(
            &plan,
            &PuzzleProgress::default(),
            &mut app.world_mut().resource_mut::<CollisionWorld>(),
        );
        for _ in 0..32 {
            app.world_mut().resource_mut::<Session>().outcome = Outcome::Caught;
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .press(KeyCode::KeyR);
            app.update();
            let collision = app.world().resource::<CollisionWorld>();
            assert_eq!(collision.static_count, collision.solids.len());
            assert_eq!(collision.static_count, plan.colliders.len() + 4);
            assert!(collision.loose_boxes.is_empty());
            for door in MysteryDoor::ALL {
                assert!(collision.solids.contains(&Aabb::from_center(
                    door.center(),
                    Vec3::new(2.25, 2.6, 0.22)
                )));
            }
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .reset_all();
            // No physics system is installed: this is the previously crashing
            // high-frame-rate gap between R and the next FixedUpdate.
            app.update();
        }
        assert_eq!(app.world().resource::<Session>().attempts, 33);
    }
    #[test]
    fn trolley_path_closes_without_corner_jumps() {
        for boundary in [0., 33., 56.6, 89.6, 113.2] {
            let a = train_pose(boundary - 0.001).0;
            let b = train_pose(boundary + 0.001).0;
            assert!(a.distance(b) < 0.003, "jump at {boundary}: {a:?} to {b:?}");
        }
    }
    fn puzzle_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<HousePlan>()
            .init_resource::<CollisionWorld>()
            .init_resource::<Session>()
            .init_resource::<Inventory>()
            .init_resource::<PuzzleProgress>()
            .init_resource::<ButtonInput<KeyCode>>()
            .add_message::<SoundCue>()
            .add_systems(
                Update,
                (rebuild_collisions, interact, ladder_controls).chain(),
            );
        app.world_mut().spawn(Player::new(PLAYER_SPAWN));
        for (kind, pos) in [
            (ItemKind::BlueKey, Vec3::new(-4.85, 1.1, 5.55)),
            (ItemKind::Crowbar, Vec3::new(-8., 4.85, -6.9)),
            (ItemKind::GreenKey, Vec3::new(-8., 8.3, -7.4)),
            (ItemKind::RedKey, Vec3::new(-10.4, 15.4, -8.6)),
        ] {
            app.world_mut().spawn((
                Pickup {
                    kind,
                    home: pos,
                    velocity: Vec3::ZERO,
                    held: false,
                },
                Transform::from_translation(pos),
                Visibility::Inherited,
            ));
        }
        app
    }
    fn interact_at(app: &mut App, feet: Vec3, target: Vec3) {
        let mut query = app.world_mut().query::<&mut Player>();
        {
            let mut p = query.single_mut(app.world_mut()).expect("one test player");
            p.pos = feet;
            p.vel = Vec3::ZERO;
            let direction = (target - p.eye()).normalize();
            p.yaw = (-direction.x).atan2(-direction.z);
            p.pitch = direction.y.asin();
        }
        {
            let mut keys = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            keys.reset_all();
            keys.press(KeyCode::KeyE);
        }
        app.update();
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .reset_all();
    }
    #[test]
    fn real_ecs_interactions_complete_all_keys_tower_and_basement() {
        let mut app = puzzle_app();
        interact_at(
            &mut app,
            Vec3::new(-4.85, FLOOR_Y, 7.2),
            Vec3::new(-4.85, 1.1, 5.55),
        );
        assert!(app.world().resource::<Inventory>().blue_key);
        interact_at(
            &mut app,
            Vec3::new(-8., 3.72, -5.2),
            Vec3::new(-8., 4.85, -6.9),
        );
        assert!(app.world().resource::<Inventory>().crowbar);
        interact_at(
            &mut app,
            Vec3::new(-8., 7.32, -2.5),
            MysteryDoor::Archive.center() + Vec3::Z * 0.5,
        );
        assert!(app.world().resource::<PuzzleProgress>().archive_open);
        interact_at(
            &mut app,
            Vec3::new(-8., 7.32, -5.8),
            Vec3::new(-8., 8.3, -7.4),
        );
        assert!(app.world().resource::<Inventory>().green_key);
        interact_at(
            &mut app,
            Vec3::new(-8., 14.52, -2.5),
            MysteryDoor::Greenhouse.center() + Vec3::Z * 0.5,
        );
        assert!(app.world().resource::<PuzzleProgress>().greenhouse_open);
        interact_at(
            &mut app,
            Vec3::new(-10.4, 14.52, -7.),
            Vec3::new(-10.4, 15.4, -8.6),
        );
        assert!(app.world().resource::<Inventory>().red_key);
        interact_at(
            &mut app,
            Vec3::new(2., 18.12, -2.),
            MysteryDoor::Tower.center() + Vec3::Z * 0.5,
        );
        assert!(app.world().resource::<PuzzleProgress>().tower_open);
        interact_at(
            &mut app,
            Vec3::new(2., 18.12, -5.2),
            Vec3::new(0.85, 19.2, -6.5),
        );
        {
            let mut q = app.world_mut().query::<&Player>();
            assert!((q.single(app.world()).expect("player").pos.y - 21.72).abs() < 0.01);
        }
        interact_at(
            &mut app,
            Vec3::new(1.8, 21.72, -6.2),
            WINDMILL_SWITCH + Vec3::Z * 0.5,
        );
        assert!(app.world().resource::<PuzzleProgress>().windmill_stopped);
        interact_at(
            &mut app,
            Vec3::new(-8., 14.52, -5.8),
            GENERATOR + Vec3::Z * 0.85,
        );
        assert!(app.world().resource::<PuzzleProgress>().power_off);
        interact_at(
            &mut app,
            Vec3::new(-8.8, FLOOR_Y, -6.8),
            FREEZER + Vec3::Z * 0.9,
        );
        assert!(app.world().resource::<Inventory>().keycard);
        for _ in 0..2 {
            interact_at(
                &mut app,
                Vec3::new(-8., FLOOR_Y, -3.3),
                BASEMENT_DOOR + Vec3::Z * 0.45,
            );
        }
        assert!(app.world().resource::<PuzzleProgress>().basement_open);
        assert!(app.world().resource::<Session>().outcome == Outcome::Exploring);
    }
    #[test]
    fn opening_a_keyed_door_removes_its_collision_on_the_next_tick() {
        let mut app = puzzle_app();
        app.update();
        let center = MysteryDoor::Archive.center();
        assert!(!line_of_sight(
            center + Vec3::Z,
            center - Vec3::Z,
            &app.world().resource::<CollisionWorld>().solids
        ));
        app.world_mut().resource_mut::<Inventory>().blue_key = true;
        interact_at(&mut app, Vec3::new(-8., 7.32, -2.5), center + Vec3::Z * 0.5);
        app.update();
        assert!(line_of_sight(
            center + Vec3::Z,
            center - Vec3::Z,
            &app.world().resource::<CollisionWorld>().solids
        ));
    }
    #[test]
    fn interactions_cannot_reach_through_walls() {
        let p = Player::new(Vec3::ZERO);
        let target = p.eye() - Vec3::Z * 2.;
        assert!(can_interact(&p, target, &[]));
        let wall = Aabb::from_center(p.eye() - Vec3::Z, Vec3::new(3., 3., 0.2));
        assert!(!can_interact(&p, target, &[wall]));
        assert!(!can_interact(&p, p.eye() + Vec3::Z, &[]));
    }
}
