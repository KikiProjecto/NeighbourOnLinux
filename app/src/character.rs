//! Original procedural character with shoulder/hip pivots and movement-driven gait.
use crate::{
    ai::{AlertState, Neighbor},
    audio::SoundCue,
    game::{Art, GameSet, Outcome, Session},
    motion::MotionHistory,
    player::Player,
    world::{NEIGHBOR_SPAWN, Paint, Shape},
};
use bevy::{ecs::system::SystemParam, prelude::*};

pub struct CharacterPlugin;
impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Pose>()
            .add_systems(Update, animate.after(GameSet::Presentation));
    }
}
#[derive(Resource)]
struct Pose {
    stride: f32,
    strength: f32,
    age: f32,
    last: Vec3,
    step: i32,
    attempt: u32,
}
impl Default for Pose {
    fn default() -> Self {
        Self {
            stride: 0.,
            strength: 0.,
            age: 0.,
            last: NEIGHBOR_SPAWN,
            step: 0,
            attempt: 1,
        }
    }
}
#[derive(Component)]
struct Joint {
    rest: Transform,
    kind: JointKind,
}
#[derive(Clone, Copy)]
enum JointKind {
    Body,
    Head,
    Arm(f32),
    Leg(f32),
    Eyelid,
}
fn joint(commands: &mut Commands, parent: Entity, position: Vec3, kind: JointKind) -> Entity {
    let rest = Transform::from_translation(position);
    let id = commands
        .spawn((rest, Visibility::Inherited, Joint { rest, kind }))
        .id();
    commands.entity(parent).add_child(id);
    id
}

pub(crate) fn spawn_neighbor(commands: &mut Commands, art: &Art) {
    let root = commands
        .spawn((
            Name::new("Peterson / procedural articulated character"),
            Neighbor::default(),
            MotionHistory::new(NEIGHBOR_SPAWN),
            Transform::from_translation(NEIGHBOR_SPAWN),
            Visibility::Inherited,
        ))
        .id();
    let body = joint(commands, root, Vec3::Y * 0.9, JointKind::Body);
    art.child(
        commands,
        body,
        Vec3::new(0., 0.24, 0.),
        Vec3::new(0.46, 0.47, 0.28),
        Paint::Vest,
        Shape::Sphere,
    );
    art.child(
        commands,
        body,
        Vec3::new(0., -0.02, 0.),
        Vec3::new(0.75, 0.16, 0.46),
        Paint::Pants,
        Shape::Box,
    );
    art.child(
        commands,
        body,
        Vec3::new(0., 0.01, 0.245),
        Vec3::new(0.72, 0.07, 0.025),
        Paint::DarkWood,
        Shape::Box,
    );
    art.child(
        commands,
        body,
        Vec3::new(0., 0.01, 0.267),
        Vec3::new(0.1, 0.08, 0.03),
        Paint::Metal,
        Shape::Box,
    );
    art.child(
        commands,
        body,
        Vec3::new(0., 0.61, 0.),
        Vec3::new(0.16, 0.16, 0.15),
        Paint::Skin,
        Shape::Cylinder,
    );
    for side in [-1., 1.] {
        let collar = art.child(
            commands,
            body,
            Vec3::new(side * 0.13, 0.55, 0.2),
            Vec3::new(0.24, 0.17, 0.05),
            Paint::Cream,
            Shape::Box,
        );
        commands.entity(collar).insert(
            Transform::from_xyz(side * 0.13, 0.55, 0.2)
                .with_scale(Vec3::new(0.24, 0.17, 0.05))
                .with_rotation(Quat::from_rotation_z(side * 0.5)),
        );
        let arm = joint(
            commands,
            body,
            Vec3::new(side * 0.48, 0.46, 0.),
            JointKind::Arm(-side),
        );
        art.child(
            commands,
            arm,
            Vec3::new(0., -0.15, 0.),
            Vec3::new(0.15, 0.23, 0.16),
            Paint::Cream,
            Shape::Sphere,
        );
        art.child(
            commands,
            arm,
            Vec3::new(0., -0.36, 0.),
            Vec3::new(0.27, 0.1, 0.29),
            Paint::White,
            Shape::Box,
        );
        art.child(
            commands,
            arm,
            Vec3::new(0., -0.49, 0.015),
            Vec3::new(0.115, 0.19, 0.12),
            Paint::Skin,
            Shape::Sphere,
        );
        art.child(
            commands,
            arm,
            Vec3::new(0., -0.69, 0.04),
            Vec3::new(0.14, 0.16, 0.145),
            Paint::Black,
            Shape::Sphere,
        );
        art.child(
            commands,
            arm,
            Vec3::new(-side * 0.1, -0.67, 0.12),
            Vec3::new(0.06, 0.09, 0.06),
            Paint::Black,
            Shape::Sphere,
        );
        let leg = joint(
            commands,
            root,
            Vec3::new(side * 0.22, 0.86, 0.),
            JointKind::Leg(side),
        );
        art.child(
            commands,
            leg,
            Vec3::new(0., -0.35, 0.),
            Vec3::new(0.28, 0.7, 0.34),
            Paint::Pants,
            Shape::Box,
        );
        art.child(
            commands,
            leg,
            Vec3::new(0., -0.7, 0.),
            Vec3::new(0.29, 0.08, 0.35),
            Paint::DarkWood,
            Shape::Box,
        );
        art.child(
            commands,
            leg,
            Vec3::new(0., -0.76, 0.1),
            Vec3::new(0.32, 0.18, 0.55),
            Paint::Black,
            Shape::Box,
        );
        art.child(
            commands,
            leg,
            Vec3::new(0., -0.83, 0.1),
            Vec3::new(0.34, 0.035, 0.57),
            Paint::DarkWood,
            Shape::Box,
        );
    }
    for row in 0..2 {
        for col in 0..3 {
            let x = (col as f32 - 1.) * 0.22;
            let y = 0.2 + row as f32 * 0.22;
            let id = art.child(
                commands,
                body,
                Vec3::new(x, y, 0.272),
                Vec3::new(0.15, 0.15, 0.02),
                if (row + col) % 2 == 0 {
                    Paint::Teal
                } else {
                    Paint::PaleBlue
                },
                Shape::Box,
            );
            commands.entity(id).insert(
                Transform::from_xyz(x, y, 0.272)
                    .with_scale(Vec3::new(0.15, 0.15, 0.02))
                    .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
            );
        }
    }
    let head = joint(commands, body, Vec3::new(0., 0.88, 0.), JointKind::Head);
    art.child(
        commands,
        head,
        Vec3::ZERO,
        Vec3::new(0.32, 0.35, 0.285),
        Paint::Skin,
        Shape::Sphere,
    );
    art.child(
        commands,
        head,
        Vec3::new(0., -0.17, 0.04),
        Vec3::new(0.27, 0.17, 0.24),
        Paint::Skin,
        Shape::Sphere,
    );
    art.child(
        commands,
        head,
        Vec3::new(0., 0.23, -0.05),
        Vec3::new(0.34, 0.19, 0.29),
        Paint::Hair,
        Shape::Sphere,
    );
    art.child(
        commands,
        head,
        Vec3::new(-0.11, 0.32, 0.04),
        Vec3::new(0.21, 0.11, 0.24),
        Paint::Hair,
        Shape::Sphere,
    );
    art.child(
        commands,
        head,
        Vec3::new(0., -0.01, 0.29),
        Vec3::new(0.105, 0.13, 0.16),
        Paint::Skin,
        Shape::Sphere,
    );
    art.child(
        commands,
        head,
        Vec3::new(0., -0.2, 0.24),
        Vec3::new(0.15, 0.025, 0.025),
        Paint::DarkWood,
        Shape::Box,
    );
    for side in [-1., 1.] {
        art.child(
            commands,
            head,
            Vec3::new(side * 0.31, -0.01, -0.01),
            Vec3::new(0.075, 0.12, 0.08),
            Paint::Skin,
            Shape::Sphere,
        );
        art.child(
            commands,
            head,
            Vec3::new(side * 0.13, 0.07, 0.25),
            Vec3::new(0.095, 0.07, 0.045),
            Paint::White,
            Shape::Sphere,
        );
        art.child(
            commands,
            head,
            Vec3::new(side * 0.13, 0.07, 0.291),
            Vec3::new(0.034, 0.043, 0.015),
            Paint::Teal,
            Shape::Sphere,
        );
        art.child(
            commands,
            head,
            Vec3::new(side * 0.13, 0.07, 0.304),
            Vec3::new(0.016, 0.03, 0.008),
            Paint::Black,
            Shape::Sphere,
        );
        let lid = joint(
            commands,
            head,
            Vec3::new(side * 0.13, 0.13, 0.29),
            JointKind::Eyelid,
        );
        art.child(
            commands,
            lid,
            Vec3::ZERO,
            Vec3::new(0.1, 0.055, 0.025),
            Paint::Skin,
            Shape::Sphere,
        );
        let brow = art.child(
            commands,
            head,
            Vec3::new(side * 0.14, 0.19, 0.26),
            Vec3::new(0.19, 0.055, 0.06),
            Paint::Hair,
            Shape::Box,
        );
        commands.entity(brow).insert(
            Transform::from_xyz(side * 0.14, 0.19, 0.26)
                .with_scale(Vec3::new(0.19, 0.055, 0.06))
                .with_rotation(Quat::from_rotation_z(side * 0.22)),
        );
        let mustache = art.child(
            commands,
            head,
            Vec3::new(side * 0.09, -0.12, 0.28),
            Vec3::new(0.13, 0.055, 0.06),
            Paint::Hair,
            Shape::Sphere,
        );
        commands.entity(mustache).insert(
            Transform::from_xyz(side * 0.09, -0.12, 0.28)
                .with_scale(Vec3::new(0.13, 0.055, 0.06))
                .with_rotation(Quat::from_rotation_z(side * 0.15)),
        );
    }
}

#[derive(SystemParam)]
struct AnimationClock<'w> {
    time: Res<'w, Time>,
    fixed: Res<'w, Time<Fixed>>,
}
fn animate(
    clock: AnimationClock,
    session: Res<Session>,
    mut pose: ResMut<Pose>,
    players: Query<&Player>,
    mut neighbors: Query<(&Neighbor, &MotionHistory, &mut Transform)>,
    mut joints: Query<(&Joint, &mut Transform), Without<Neighbor>>,
    mut cues: MessageWriter<SoundCue>,
) {
    if session.paused || session.outcome != Outcome::Exploring {
        return;
    }
    let Ok((neighbor, history, mut root)) = neighbors.single_mut() else {
        return;
    };
    if pose.attempt != session.attempts {
        *pose = Pose {
            attempt: session.attempts,
            ..default()
        };
    }
    let dt = clock.time.delta_secs();
    pose.age += dt;
    let position = history.position(neighbor.body.pos, clock.fixed.overstep_fraction());
    let distance = (position - pose.last).xz().length().min(0.5);
    pose.last = position;
    pose.stride += distance * 3.2;
    let target_strength = if neighbor.walking && distance > 0.0001 {
        1.
    } else {
        0.
    };
    pose.strength += (target_strength - pose.strength) * (1. - (-10. * dt).exp());
    root.translation = position;
    let facing = Quat::from_rotation_y(neighbor.facing.x.atan2(neighbor.facing.z));
    root.rotation = root.rotation.slerp(facing, 1. - (-12. * dt).exp());
    let step = (pose.stride / std::f32::consts::PI).floor() as i32;
    if step != pose.step && distance > 0.0001 && neighbor.body.on_ground {
        cues.write(SoundCue::NeighborStep(neighbor.body.pos));
    }
    pose.step = step;
    let alert = if neighbor.state == AlertState::Chase {
        1.
    } else {
        0.
    };
    let look = players.single().ok().map_or(0., |p| {
        let delta = p.eye() - neighbor.body.eye();
        let local = root.rotation.inverse() * delta;
        local.x.atan2(local.z).clamp(-0.45, 0.45)
    });
    for (joint, mut tf) in &mut joints {
        *tf = joint.rest;
        match joint.kind {
            JointKind::Body => {
                tf.translation.y += (pose.age * 1.7).sin() * 0.009
                    + pose.stride.sin().abs() * pose.strength * 0.025;
                tf.rotate_x(alert * 0.09);
            }
            JointKind::Head => {
                tf.rotate_y(
                    look * if neighbor.state == AlertState::Patrol {
                        0.15
                    } else {
                        1.
                    },
                );
                tf.rotate_z((pose.age * 0.6).sin() * 0.018);
            }
            JointKind::Arm(side) => {
                tf.rotate_x(
                    pose.stride.sin() * side * pose.strength * (0.32 + alert * 0.18) - alert * 0.22,
                );
                tf.rotate_z(side * -0.04);
            }
            JointKind::Leg(side) => {
                tf.rotate_x(pose.stride.sin() * side * pose.strength * (0.3 + alert * 0.15))
            }
            JointKind::Eyelid => {
                let blink = ((pose.age.rem_euclid(4.7) - 4.5) / 0.1).abs();
                tf.translation.y -= (1. - blink).max(0.) * 0.06;
                tf.scale.y += (1. - blink).max(0.);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn articulated_pose_tracks_motion_and_freezes_on_pause() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f32(1. / 60.),
            ))
            .add_systems(FixedUpdate, crate::motion::record_movement)
            .insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
                std::time::Duration::from_secs_f32(1. / 60.),
            ))
            .init_resource::<Session>()
            .insert_resource(Art::test_fixture())
            .add_message::<SoundCue>()
            .add_plugins(CharacterPlugin)
            .add_systems(Startup, |mut commands: Commands, art: Res<Art>| {
                spawn_neighbor(&mut commands, &art);
            });
        app.update();
        app.update();
        let id = app
            .world_mut()
            .query_filtered::<Entity, With<Neighbor>>()
            .single(app.world())
            .unwrap();
        for _ in 0..15 {
            let mut n = app.world_mut().get_mut::<Neighbor>(id).unwrap();
            n.body.pos.z += 0.03;
            n.walking = true;
            app.update();
        }
        assert!(app.world().resource::<Pose>().strength > 0.5);
        let transforms: Vec<_> = app
            .world_mut()
            .query::<(Entity, &Joint, &Transform)>()
            .iter(app.world())
            .map(|(id, _, tf)| (id, *tf))
            .collect();
        assert!(transforms.len() >= 8);
        app.world_mut().resource_mut::<Session>().paused = true;
        for _ in 0..30 {
            app.update();
        }
        for (id, tf) in transforms {
            assert_eq!(*app.world().get::<Transform>(id).unwrap(), tf);
        }
    }
}
