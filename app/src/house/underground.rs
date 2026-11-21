//! Underground architecture lives with the house, sharing collision and navigation data.
use crate::{
    basement::{
        self, CELL, ENTRY, FLOOR, HEIGHT, Labyrinth, ROOMS, STAIR_CELL, WIDTH, cell_center,
    },
    world::{FLOOR_Y, HousePlan, Paint, Paint::*},
};
use bevy::prelude::*;

const HOLE_MIN: Vec2 = Vec2::new(-9.2, -9.6);
const HOLE_MAX: Vec2 = Vec2::new(-6.8, -5.1);

/// Four solid rectangles leave an actual stair opening through soil, floor and ceiling.
pub(super) fn slab_with_entrance(
    h: &mut HousePlan,
    name: &'static str,
    center: Vec3,
    size: Vec3,
    paint: Paint,
) {
    let min = center - size * 0.5;
    let max = center + size * 0.5;
    for (x0, x1, z0, z1) in [
        (min.x, HOLE_MIN.x, min.z, max.z),
        (HOLE_MAX.x, max.x, min.z, max.z),
        (HOLE_MIN.x, HOLE_MAX.x, min.z, HOLE_MIN.y),
        (HOLE_MIN.x, HOLE_MAX.x, HOLE_MAX.y, max.z),
    ] {
        h.block(
            name,
            Vec3::new((x0 + x1) * 0.5, center.y, (z0 + z1) * 0.5),
            Vec3::new(x1 - x0, size.y, z1 - z0),
            paint,
            true,
        );
    }
}

pub(super) fn build(h: &mut HousePlan) {
    let maze = Labyrinth::default();
    let center = cell_center(5, 5);
    let footprint = WIDTH as f32 * CELL;
    h.room(
        "Underground · maintenance labyrinth",
        center + Vec3::Y * HEIGHT * 0.5,
        Vec3::new(footprint, HEIGHT, footprint),
    );
    for room in ROOMS {
        h.room(
            room.name,
            room.center() + Vec3::Y * HEIGHT * 0.5,
            Vec3::new(CELL * 2. - 0.3, HEIGHT, CELL * 2. - 0.3),
        );
    }
    for z in 0..WIDTH {
        for x in 0..WIDTH {
            let p = cell_center(x, z);
            h.block(
                "Labyrinth concrete floor",
                p - Vec3::Y * 0.15,
                Vec3::new(CELL, 0.3, CELL),
                if (x + z) % 3 == 0 { Metal } else { Concrete },
                true,
            );
            // Painted expansion joints, rather than hundreds of additional collision shapes.
            h.deco(
                "Underground floor joint",
                p + Vec3::new(0., 0.006, -CELL * 0.5 + 0.12),
                Vec3::new(CELL - 0.24, 0.012, 0.035),
                Black,
            );
            if z == 0 {
                wall(h, p + Vec3::new(0., 0., -CELL * 0.5), false, (x + z) % 3);
            }
            if x == 0 {
                wall(h, p + Vec3::new(-CELL * 0.5, 0., 0.), true, (x + z) % 3);
            }
            if maze.passages[z][x] & 2 == 0 {
                wall(h, p + Vec3::new(CELL * 0.5, 0., 0.), true, (x + z) % 3);
            }
            if maze.passages[z][x] & 4 == 0 {
                wall(h, p + Vec3::new(0., 0., CELL * 0.5), false, (x + z) % 3);
            }
            // Alternating overhead pipe runs give junctions scale and a direction to follow.
            if z % 3 == 1 && (x, z) != STAIR_CELL {
                h.pipe(
                    p + Vec3::new(-CELL * 0.5, 2.88, -0.65),
                    p + Vec3::new(CELL * 0.5, 2.88, -0.65),
                    0.065,
                    Metal,
                );
                h.deco(
                    "Pipe ceiling bracket",
                    p + Vec3::new(0., 3.05, -0.65),
                    Vec3::new(0.08, 0.24, 0.1),
                    DarkWood,
                );
            }
            if (x + z) % 3 == 0 && (x, z) != STAIR_CELL {
                h.deco(
                    "Tunnel emergency luminaire",
                    p + Vec3::new(0., 3.02, 0.),
                    Vec3::new(0.5, 0.11, 0.28),
                    Glow,
                );
            }
        }
    }
    slab_with_entrance(
        h,
        "Labyrinth concrete ceiling",
        center + Vec3::Y * (HEIGHT + 0.15),
        Vec3::new(footprint + 0.3, 0.3, footprint + 0.3),
        Concrete,
    );
    entrance(h);
    for (index, room) in ROOMS.into_iter().enumerate() {
        furnish(h, index, room.center());
    }
    for (index, room) in basement::CIRCUIT_ROOMS.into_iter().enumerate() {
        let p = basement::circuit_position(index);
        h.block(
            "Basement circuit cabinet",
            p - Vec3::Z * 0.17,
            Vec3::new(1.1, 1.1, 0.34),
            Metal,
            true,
        );
        h.deco(
            "Circuit panel inset",
            p + Vec3::Z * 0.015,
            Vec3::new(0.84, 0.85, 0.04),
            Black,
        );
        h.deco(
            "Circuit cable legend",
            p + Vec3::new(-0.38, 0., 0.055),
            Vec3::new(0.08, 0.8, 0.04),
            [Orange, Teal, Purple][index],
        );
        let destination = (ROOMS[room].x, ROOMS[room].z);
        let mut points: Vec<_> = maze
            .path(destination)
            .into_iter()
            .map(|(x, z)| cell_center(x, z))
            .collect();
        // Continue from the room's first cell to its breaker without crossing furniture.
        let corner = ROOMS[room].center() + Vec3::new(0., 0., -CELL * 0.5);
        points.push(corner);
        points.push(p.with_y(FLOOR));
        for pair in points.windows(2) {
            let offset = Vec3::new(index as f32 * 0.1 - 0.1, 0.025, index as f32 * 0.1 - 0.1);
            h.beam(
                "Colored basement guide cable",
                pair[0] + offset,
                pair[1] + offset,
                0.045,
                [Orange, Teal, Purple][index],
            );
        }
    }
    let board = basement::map_position();
    h.deco(
        "Maintenance route board",
        board - Vec3::Z * 0.1,
        Vec3::new(1.2, 0.9, 0.1),
        Cream,
    );
    for i in 0..3 {
        h.deco(
            "Maintenance map cable legend",
            board + Vec3::new(-0.33 + i as f32 * 0.33, 0., -0.035),
            Vec3::new(0.13, 0.55, 0.025),
            [Orange, Teal, Purple][i],
        );
    }
    let hatch = basement::exit_position();
    h.block(
        "Service exit blast door",
        hatch - Vec3::Z * 0.13,
        Vec3::new(1.9, 2.5, 0.2),
        Teal,
        true,
    );
    for x in [-0.7, 0.7] {
        h.deco(
            "Exit door reinforcement",
            hatch + Vec3::new(x, 0., 0.),
            Vec3::new(0.08, 2.3, 0.08),
            Metal,
        );
    }
    h.deco(
        "Service exit handle",
        hatch + Vec3::new(0., -0.15, 0.05),
        Vec3::new(0.4, 0.08, 0.12),
        Yellow,
    );
    navigation(h, &maze);
}

fn wall(h: &mut HousePlan, p: Vec3, vertical: bool, sector: usize) {
    let size = if vertical {
        Vec3::new(0.24, HEIGHT, CELL + 0.24)
    } else {
        Vec3::new(CELL + 0.24, HEIGHT, 0.24)
    };
    h.block(
        "Labyrinth retaining wall",
        p + Vec3::Y * HEIGHT * 0.5,
        size,
        Concrete,
        true,
    );
    h.deco(
        "Underground painted wall band",
        p + Vec3::Y * 0.68,
        Vec3::new(size.x + 0.012, 0.75, size.z + 0.012),
        [Teal, DarkWood, PaleBlue][sector],
    );
    h.deco(
        "Underground wall footing",
        p + Vec3::Y * 0.12,
        Vec3::new(size.x + 0.04, 0.24, size.z + 0.04),
        DarkWood,
    );
}

fn entrance(h: &mut HousePlan) {
    for i in 0..18 {
        let top = FLOOR_Y - (i + 1) as f32 * 0.24;
        let z = -5.1 - (i as f32 + 0.5) * 0.25;
        h.block(
            "Basement descending stair",
            Vec3::new(-8., (top + FLOOR) * 0.5, z),
            Vec3::new(2.4, (top - FLOOR).max(0.02), 0.25),
            Concrete,
            true,
        );
        h.deco(
            "Basement step safety nosing",
            Vec3::new(-8., top + 0.008, z + 0.1),
            Vec3::new(2.2, 0.016, 0.035),
            Yellow,
        );
    }
    // Side retaining walls isolate the stair volume from the neighboring maze cell.
    for x in [-9.33, -6.67] {
        h.block(
            "Stairwell retaining wall",
            Vec3::new(x, -2.04, -7.35),
            Vec3::new(0.26, 4.32, 4.5),
            Concrete,
            true,
        );
        h.pipe(
            Vec3::new(x, -0.1, -5.3),
            Vec3::new(x, -3.6, -9.25),
            0.055,
            Metal,
        );
    }
    h.room(
        "Underground · entrance stairs",
        Vec3::new(-8., -1.8, -7.35),
        Vec3::new(2.4, 5.4, 4.5),
    );
}

fn bench(h: &mut HousePlan, p: Vec3, width: f32, depth: f32) {
    h.block(
        "Underground workbench",
        p + Vec3::Y * 0.92,
        Vec3::new(width, 0.16, depth),
        DarkWood,
        true,
    );
    for x in [-1., 1.] {
        for z in [-1., 1.] {
            h.deco(
                "Workbench leg",
                p + Vec3::new(x * (width * 0.5 - 0.12), 0.43, z * (depth * 0.5 - 0.12)),
                Vec3::new(0.12, 0.86, 0.12),
                Metal,
            );
        }
    }
}
fn tank(h: &mut HousePlan, p: Vec3, paint: Paint) {
    h.pieces.push(crate::world::Piece {
        name: "Underground pressure vessel",
        transform: Transform::from_translation(p + Vec3::Y * 1.13)
            .with_scale(Vec3::new(0.65, 2.05, 0.65)),
        paint,
        shape: crate::world::Shape::Cylinder,
    });
    h.colliders.push(crate::world::Aabb::from_center(
        p + Vec3::Y * 1.13,
        Vec3::new(1.3, 2.05, 1.3),
    ));
    for y in [0.45, 1.8] {
        h.deco(
            "Pressure vessel strap",
            p + Vec3::Y * y,
            Vec3::new(1.33, 0.1, 1.33),
            Metal,
        );
    }
    h.pipe(
        p + Vec3::new(0., 2.25, 0.),
        p + Vec3::new(0., 2.85, 0.),
        0.09,
        Metal,
    );
    h.sphere(
        "Pressure gauge",
        p + Vec3::new(0., 1.65, 0.66),
        Vec3::new(0.18, 0.18, 0.04),
        Cream,
    );
    h.deco(
        "Pressure gauge needle",
        p + Vec3::new(0.025, 1.68, 0.71),
        Vec3::new(0.025, 0.2, 0.02),
        Red,
    );
}
fn shelf(h: &mut HousePlan, p: Vec3, books: bool) {
    h.block(
        "Underground shelving frame",
        p + Vec3::Y * 1.05,
        Vec3::new(0.65, 2.1, 2.4),
        Metal,
        true,
    );
    for row in 0..4 {
        let y = 0.25 + row as f32 * 0.47;
        h.deco(
            "Underground shelf",
            p + Vec3::Y * y,
            Vec3::new(0.8, 0.08, 2.5),
            DarkWood,
        );
        for column in 0..5 {
            h.deco(
                if books {
                    "Archive files"
                } else {
                    "Maintenance spare parts"
                },
                p + Vec3::new(0.1, y + 0.19, -0.95 + column as f32 * 0.47),
                Vec3::new(0.45, 0.3, 0.3),
                if books {
                    [Cream, DarkWood, Red][column % 3]
                } else {
                    Metal
                },
            );
        }
    }
}
fn furnish(h: &mut HousePlan, index: usize, p: Vec3) {
    // All furniture is outside the four center-to-center navigation lanes in each chamber.
    let accent = [
        Orange, Yellow, Teal, Red, Blue, PaleBlue, Cream, Purple, Teal, Glass, Yellow, Leaf,
    ][index];
    h.deco(
        "Chamber painted floor border",
        p + Vec3::Y * 0.013,
        Vec3::new(1.7, 0.025, 1.7),
        accent,
    );
    h.deco(
        "Chamber ceiling fixture",
        p + Vec3::Y * 3.02,
        Vec3::new(1.4, 0.1, 0.35),
        Glow,
    );
    match index {
        0 => {
            for x in [-3.4, 3.4] {
                tank(h, p + Vec3::new(x, 0., -3.35), Orange);
            }
            for x in [-3.4, 3.4] {
                h.pipe(
                    p + Vec3::new(x, 2.8, -3.35),
                    p + Vec3::new(0., 2.8, -3.35),
                    0.09,
                    Metal,
                );
            }
            bench(h, p, 1.5, 1.1);
            h.deco(
                "Boiler maintenance log",
                p + Vec3::Y * 1.025,
                Vec3::new(0.65, 0.03, 0.45),
                Cream,
            );
        }
        1 => {
            bench(h, p, 1.8, 1.25);
            shelf(h, p + Vec3::new(-3.7, 0., 0.), false);
            for i in 0..5 {
                h.deco(
                    "Workshop tools",
                    p + Vec3::new(-0.55 + i as f32 * 0.26, 1.04, 0.),
                    Vec3::new(0.08, 0.06, 0.5),
                    Metal,
                );
            }
        }
        2 | 4 => {
            for x in [-3.4, 3.4] {
                tank(
                    h,
                    p + Vec3::new(x, 0., -3.35),
                    if index == 2 { Teal } else { Blue },
                );
            }
            h.deco(
                "Still cistern water",
                p + Vec3::Y * 0.04,
                Vec3::new(1.7, 0.025, 1.7),
                Glass,
            );
            for x in [-0.95, 0.95] {
                h.deco(
                    "Cistern basin rim",
                    p + Vec3::new(x, 0.14, 0.),
                    Vec3::new(0.12, 0.28, 1.95),
                    Concrete,
                );
            }
            for z in [-0.95, 0.95] {
                h.deco(
                    "Cistern basin rim",
                    p + Vec3::new(0., 0.14, z),
                    Vec3::new(1.95, 0.28, 0.12),
                    Concrete,
                );
            }
        }
        3 => {
            for x in [-3.55, 3.55] {
                h.block(
                    "Underground transformer",
                    p + Vec3::new(x, 0.85, -3.5),
                    Vec3::new(1.15, 1.7, 1.35),
                    Red,
                    true,
                );
                for y in 0..6 {
                    h.deco(
                        "Transformer cooling fins",
                        p + Vec3::new(x, 0.35 + y as f32 * 0.22, -2.79),
                        Vec3::new(0.9, 0.08, 0.08),
                        Black,
                    );
                }
            }
            h.pipe(
                p + Vec3::new(-3.5, 2.8, -3.4),
                p + Vec3::new(3.5, 2.8, -3.4),
                0.1,
                Black,
            );
        }
        5 => {
            for x in [-3.65, 3.65] {
                h.block(
                    "Cold storage cabinet",
                    p + Vec3::new(x, 1.15, 0.),
                    Vec3::new(0.85, 2.3, 1.7),
                    PaleBlue,
                    true,
                );
                h.deco(
                    "Cold storage latch",
                    p + Vec3::new(x.signum() * 3.18, 1.2, 0.),
                    Vec3::new(0.1, 0.35, 0.08),
                    Metal,
                );
            }
            h.deco(
                "Cold room vent",
                p + Vec3::new(0., 2.5, -4.18),
                Vec3::new(1.3, 0.7, 0.08),
                Black,
            );
            for i in 0..6 {
                h.deco(
                    "Vent slat",
                    p + Vec3::new(0., 2.23 + i as f32 * 0.1, -4.1),
                    Vec3::new(1.2, 0.035, 0.06),
                    Metal,
                );
            }
        }
        6 => {
            shelf(h, p + Vec3::new(-3.7, 0., 0.), true);
            shelf(h, p + Vec3::new(3.7, 0., 0.), true);
            bench(h, p, 1.8, 1.2);
            for i in 0..3 {
                h.deco(
                    "Basement records",
                    p + Vec3::new(i as f32 * 0.18 - 0.2, 1.01 + i as f32 * 0.015, 0.),
                    Vec3::new(0.8, 0.02, 0.52),
                    Cream,
                );
            }
        }
        7 => {
            bench(h, p, 1.8, 1.2);
            h.deco(
                "Observation monitor",
                p + Vec3::new(0., 1.35, 0.),
                Vec3::new(0.95, 0.65, 0.45),
                Black,
            );
            h.deco(
                "Observation screen",
                p + Vec3::new(0., 1.35, 0.235),
                Vec3::new(0.75, 0.46, 0.02),
                Teal,
            );
            for i in 0..3 {
                h.deco(
                    "Surveillance recording",
                    p + Vec3::new(-0.28 + i as f32 * 0.28, 1.4, 0.25),
                    Vec3::new(0.035, 0.25, 0.02),
                    Glow,
                );
            }
        }
        8 => {
            for x in [-3.4, 3.4] {
                h.block(
                    "Ventilation duct",
                    p + Vec3::new(x, 2.35, -3.35),
                    Vec3::new(1.2, 1.1, 1.2),
                    Metal,
                    true,
                );
            }
            h.pipe(
                p + Vec3::new(-3.4, 2.6, -3.35),
                p + Vec3::new(3.4, 2.6, -3.35),
                0.25,
                Metal,
            );
            h.deco(
                "Ventilation fan housing",
                p + Vec3::new(0., 2.15, 3.95),
                Vec3::new(1.6, 1.6, 0.25),
                Black,
            );
        }
        9 => {
            bench(h, p, 1.8, 1.4);
            for x in [-0.5, 0., 0.5] {
                h.pipe(
                    p + Vec3::new(x, 1.05, 0.),
                    p + Vec3::new(x, 1.55, 0.),
                    0.1,
                    Glass,
                );
                h.sphere(
                    "Laboratory flask",
                    p + Vec3::new(x, 1.12, 0.),
                    Vec3::new(0.15, 0.16, 0.15),
                    [Teal, Purple, Orange][((x + 0.5) * 2.) as usize],
                );
            }
            shelf(h, p + Vec3::new(-3.7, 0., 0.), false);
            h.deco(
                "Laboratory wiring diagram",
                p + Vec3::new(0., 1.8, -4.18),
                Vec3::new(1.7, 1.1, 0.06),
                Cream,
            );
            for i in 0..3 {
                h.deco(
                    "Circuit diagram",
                    p + Vec3::new(-0.5 + i as f32 * 0.5, 1.8, -4.14),
                    Vec3::new(0.17, 0.7, 0.02),
                    [Orange, Teal, Purple][i],
                );
            }
        }
        10 => {
            bench(h, p, 1.8, 1.4);
            h.deco(
                "Powered key case base",
                p + Vec3::Y * 1.035,
                Vec3::new(0.9, 0.1, 0.7),
                Black,
            );
            for x in [-3.65, 3.65] {
                h.block(
                    "Evidence locker",
                    p + Vec3::new(x, 1.15, 0.),
                    Vec3::new(0.85, 2.3, 1.8),
                    Metal,
                    true,
                );
                for y in [0.6, 1.5] {
                    h.deco(
                        "Evidence locker seal",
                        p + Vec3::new(x.signum() * 3.18, y, 0.),
                        Vec3::new(0.08, 0.12, 0.75),
                        Yellow,
                    );
                }
            }
        }
        _ => {
            bench(h, p + Vec3::new(0., 0., 3.6), 1.6, 0.8);
            h.deco(
                "Exit emergency lamp",
                p + Vec3::new(0., 2.9, -3.8),
                Vec3::new(1.6, 0.25, 0.15),
                Glow,
            );
        }
    }
}

fn navigation(h: &mut HousePlan, maze: &Labyrinth) {
    let kitchen = h
        .nearest_node(Vec3::new(-3., FLOOR_Y, -6.))
        .expect("house kitchen navigation exists");
    let approach = h.node(Vec3::new(-3., FLOOR_Y, -3.5));
    h.link(kitchen, approach);
    let door = h.node(Vec3::new(-8., FLOOR_Y, -3.5));
    h.link(approach, door);
    let mut nodes = [[0; WIDTH]; WIDTH];
    for (z, row) in nodes.iter_mut().enumerate() {
        for (x, node) in row.iter_mut().enumerate() {
            *node = h.node(if (x, z) == STAIR_CELL {
                Vec3::new(-8., FLOOR_Y, -4.6)
            } else {
                cell_center(x, z)
            });
        }
    }
    h.link(door, nodes[STAIR_CELL.1][STAIR_CELL.0]);
    for z in 0..WIDTH {
        for x in 0..WIDTH {
            for direction in [1, 2] {
                if (x, z) == STAIR_CELL || (x, z) == ENTRY && direction == 2 {
                    continue;
                }
                if maze.passages[z][x] & (1 << direction) != 0
                    && let Some((nx, nz)) = maze.neighbor(x, z, direction)
                {
                    h.link(nodes[z][x], nodes[nz][nx]);
                }
            }
        }
    }
    let mut previous = nodes[STAIR_CELL.1][STAIR_CELL.0];
    for i in 0..18 {
        let step = h.node(Vec3::new(
            -8.,
            FLOOR_Y - (i + 1) as f32 * 0.24,
            -5.1 - (i as f32 + 0.5) * 0.25,
        ));
        h.link(previous, step);
        previous = step;
    }
    h.link(previous, nodes[ENTRY.1][ENTRY.0]);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        player::{MoveInput, Player},
        world::{Aabb, PLAYER_SPAWN},
    };
    fn walk(player: &mut Player, target: Vec3, solids: &[Aabb]) {
        for _ in 0..700 {
            let delta = (target - player.pos).with_y(0.);
            if delta.length() < 0.08 {
                player.vel.x = 0.;
                player.vel.z = 0.;
                for _ in 0..30 {
                    player.step(
                        1. / 60.,
                        &MoveInput {
                            forward: 0.,
                            strafe: 0.,
                            jump: false,
                            run: false,
                            crouch: false,
                            look: Vec2::ZERO,
                        },
                        solids,
                    );
                }
                let on_stairs = (target.x + 8.).abs() < 0.01 && (-9.7..-5.).contains(&target.z);
                assert!(
                    (player.pos.y - target.y).abs() < if on_stairs { 0.55 } else { 0.3 },
                    "wrong landing at {:?} expected {target:?}",
                    player.pos
                );
                return;
            }
            player.yaw = (-delta.x).atan2(-delta.z);
            player.step(
                1. / 60.,
                &MoveInput {
                    forward: 1.,
                    strafe: 0.,
                    jump: false,
                    run: false,
                    crouch: false,
                    look: Vec2::ZERO,
                },
                solids,
            );
        }
        panic!("blocked at {:?} on route to {target:?}", player.pos);
    }
    #[test]
    fn entrance_stairs_allow_walking_down_and_back_up_without_jumping() {
        let plan = HousePlan::default();
        let mut player = Player::new(Vec3::new(-8., FLOOR_Y, -4.4));
        walk(&mut player, cell_center(ENTRY.0, ENTRY.1), &plan.colliders);
        walk(&mut player, Vec3::new(-8., FLOOR_Y, -4.4), &plan.colliders);
    }
    #[test]
    fn every_underground_navigation_edge_is_walkable_in_both_directions() {
        let plan = HousePlan::default();
        for (id, node) in plan
            .navigation
            .iter()
            .enumerate()
            .filter(|(_, n)| n.pos.y < 0.)
        {
            for &next in node.links.iter().filter(|&&next| next > id) {
                let target = plan.navigation[next].pos;
                let mut player = Player::new(node.pos);
                walk(&mut player, target, &plan.colliders);
                walk(&mut player, node.pos, &plan.colliders);
            }
        }
        for room in ROOMS {
            assert!(
                !plan.route(PLAYER_SPAWN, room.center()).is_empty(),
                "disconnected {}",
                room.name
            );
        }
    }
    #[test]
    fn all_underground_controls_have_a_standing_clear_approach_in_the_built_level() {
        let plan = HousePlan::default();
        let targets = [
            basement::circuit_position(0),
            basement::circuit_position(1),
            basement::circuit_position(2),
            basement::key_position(),
            basement::exit_position(),
            basement::map_position(),
            basement::notes_position(),
        ];
        for target in targets {
            let can_reach = (0..16).any(|i| {
                let angle = i as f32 * std::f32::consts::TAU / 16.;
                let mut player = Player::new(
                    target.with_y(FLOOR) + Vec3::new(angle.cos() * 1.8, 0., angle.sin() * 1.8),
                );
                if plan
                    .colliders
                    .iter()
                    .any(|solid| player.aabb().overlaps(solid))
                {
                    return false;
                }
                let direction = (target - player.eye()).normalize();
                player.yaw = (-direction.x).atan2(-direction.z);
                player.pitch = direction.y.asin();
                crate::game::can_interact(&player, target, &plan.colliders)
            });
            assert!(can_reach, "no clear approach to {target:?}");
        }
    }
}
