//! Authored Act 3 inspired blockout. Not a one-to-one map or extracted game assets.
use crate::world::{
    BASEMENT_DOOR, FLOOR_Y, FREEZER, GENERATOR, HousePlan, Paint::*, STOREY, TRAIN_SWITCH,
    WINDMILL_SWITCH,
};
use bevy::prelude::*;
mod underground;

pub fn build(h: &mut HousePlan) {
    neighborhood(h);
    main_house(h);
    interiors(h);
    impossible_additions(h);
    mystery_rooms(h);
    navigation(h);
    h.underground_start = h.pieces.len();
    underground::build(h);
}

fn neighborhood(h: &mut HousePlan) {
    underground::slab_with_entrance(
        h,
        "Neighborhood ground",
        Vec3::new(0., -0.3, 0.),
        Vec3::new(320., 0.6, 320.),
        Grass,
    );
    h.deco(
        "Friendly Court",
        Vec3::new(0., 0.015, 23.),
        Vec3::new(150., 0.03, 9.),
        Road,
    );
    for z in [17.8, 28.2] {
        h.deco(
            "Sidewalk",
            Vec3::new(0., 0.06, z),
            Vec3::new(150., 0.12, 1.8),
            Concrete,
        );
        h.deco(
            "Curb",
            Vec3::new(0., 0.08, z - 1.),
            Vec3::new(150., 0.16, 0.18),
            White,
        );
    }
    for i in -16..17 {
        h.deco(
            "Road center markings",
            Vec3::new(i as f32 * 4.2, 0.04, 23.),
            Vec3::new(1.8, 0.02, 0.12),
            Cream,
        );
    }
    h.deco(
        "Front path",
        Vec3::new(0., 0.05, 12.5),
        Vec3::new(2.6, 0.1, 9.),
        Concrete,
    );
    // Low, irregular pickets frame the yard, leaving a usable front gate.
    for side in [-1., 1.] {
        for i in 0..25 {
            let x = side * (2.2 + i as f32 * 0.68);
            let height = 1.15 + (i % 3) as f32 * 0.11;
            h.block(
                "White picket",
                Vec3::new(x, height * 0.5, 15.2),
                Vec3::new(0.14, height, 0.15),
                White,
                true,
            );
        }
        for y in [0.35, 0.85] {
            h.deco(
                "Fence rail",
                Vec3::new(side * 10.2, y, 15.2),
                Vec3::new(17., 0.12, 0.13),
                Cream,
            );
        }
        for i in 0..48 {
            h.block(
                "Side fence",
                Vec3::new(side * 19.5, 0.62, 14.6 - i as f32 * 0.65),
                Vec3::new(0.14, 1.24, 0.16),
                White,
                true,
            );
        }
    }
    h.block(
        "Mailbox post",
        Vec3::new(-3., 0.55, 16.5),
        Vec3::new(0.16, 1.1, 0.16),
        DarkWood,
        true,
    );
    h.deco(
        "Mailbox",
        Vec3::new(-3., 1.25, 16.5),
        Vec3::new(0.65, 0.5, 0.75),
        Blue,
    );
    h.deco(
        "Mailbox flag",
        Vec3::new(-2.62, 1.6, 16.5),
        Vec3::new(0.08, 0.4, 0.25),
        Red,
    );
    for (x, z, s) in [
        (-26., 3., 1.3),
        (27., -9., 1.5),
        (-28., -20., 1.8),
        (25., 15., 1.1),
        (-33., 32., 1.4),
        (35., 32., 1.5),
    ] {
        tree(h, Vec3::new(x, 0., z), s);
    }
    for (x, z) in [(-35., -5.), (36., -7.), (-33., 43.), (34., 44.)] {
        distant_house(h, x, z);
    }
    // Abandoned red car beside the house, with tires and cream window frames.
    h.block(
        "Peterson car",
        Vec3::new(15.3, 0.7, 9.3),
        Vec3::new(2.8, 0.95, 5.4),
        Red,
        true,
    );
    h.deco(
        "Car cab",
        Vec3::new(15.3, 1.5, 9.4),
        Vec3::new(2.5, 0.9, 2.7),
        Red,
    );
    h.deco(
        "Windshield",
        Vec3::new(15.3, 1.6, 10.82),
        Vec3::new(2.15, 0.65, 0.06),
        Glass,
    );
    h.deco(
        "Car bumper",
        Vec3::new(15.3, 0.52, 12.04),
        Vec3::new(2.9, 0.2, 0.16),
        Metal,
    );
    for x in [13.85, 16.75] {
        for z in [7.7, 11.] {
            h.sphere(
                "Car tire",
                Vec3::new(x, 0.5, z),
                Vec3::new(0.28, 0.55, 0.55),
                Black,
            );
        }
    }
    // Your own porch is an actual safe side of the street.
    h.room(
        "Your house · across the street",
        Vec3::new(0., 2., 36.),
        Vec3::new(23., 6., 12.),
    );
    h.block(
        "Your porch",
        Vec3::new(0., 0.15, 33.),
        Vec3::new(10., 0.3, 3.),
        Wood,
        true,
    );
    distant_house(h, 0., 41.);
    landscape(h);
}

fn landscape(h: &mut HousePlan) {
    // Low-detail backdrop beyond the walkable ground, layered through the haze.
    for (x, z, width, height) in [
        (-103., -75., 34., 14.),
        (-58., -112., 45., 18.),
        (8., -125., 43., 13.),
        (70., -112., 38., 19.),
        (110., -60., 30., 12.),
        (-115., 26., 35., 16.),
        (115., 40., 34., 15.),
        (-68., 115., 38., 14.),
        (48., 118., 46., 18.),
    ] {
        h.sphere(
            "Distant wooded ridge",
            Vec3::new(x, -3., z),
            Vec3::new(width, height, width * 0.72),
            Teal,
        );
    }
    for i in 0..14 {
        let x = -65. + i as f32 * 10.;
        tree(
            h,
            Vec3::new(x, 0., -56. - ((i * 13) % 17) as f32),
            1.25 + (i % 3) as f32 * 0.28,
        );
    }
    // Break the lawn's empty plane with planting around the path and fence.
    for side in [-1., 1.] {
        for i in 0..8 {
            let x = side * (3.6 + i as f32 * 1.65);
            let z = 14.15 - (i % 3) as f32 * 0.4;
            h.sphere(
                "Garden border shrub",
                Vec3::new(x, 0.28, z),
                Vec3::new(0.65, 0.4, 0.52),
                Leaf,
            );
            for flower in 0..3 {
                let p = Vec3::new(
                    x + (flower as f32 - 1.) * 0.22,
                    0.55 + (flower % 2) as f32 * 0.14,
                    z + 0.25,
                );
                h.pipe(p - Vec3::Y * 0.38, p, 0.018, Leaf);
                h.sphere(
                    "Garden flowers",
                    p,
                    Vec3::new(0.095, 0.065, 0.095),
                    if i % 2 == 0 { Yellow } else { Orange },
                );
            }
        }
        for i in 0..9 {
            let p = Vec3::new(
                side * (22. + (i % 2) as f32 * 1.2),
                0.12,
                -29. + i as f32 * 6.6,
            );
            h.sphere("Roadside planting", p, Vec3::new(1.3, 0.42, 0.85), Grass);
        }
    }
    for x in [-24., 24.] {
        h.pipe(
            Vec3::new(x, 0., 29.8),
            Vec3::new(x, 5.1, 29.8),
            0.075,
            Metal,
        );
        h.pipe(
            Vec3::new(x, 5.1, 29.8),
            Vec3::new(x, 5.1, 28.9),
            0.06,
            Metal,
        );
        h.sphere(
            "Streetlamp shade",
            Vec3::new(x, 5.05, 28.9),
            Vec3::new(0.3, 0.11, 0.3),
            Metal,
        );
        h.sphere(
            "Streetlamp warm glass",
            Vec3::new(x, 4.94, 28.9),
            Vec3::new(0.22, 0.06, 0.22),
            Glow,
        );
    }
}
fn distant_house(h: &mut HousePlan, x: f32, z: f32) {
    h.block(
        "Raven Brooks house",
        Vec3::new(x, 2.7, z),
        Vec3::new(11., 5.4, 8.),
        Cream,
        true,
    );
    roof(h, Vec3::new(x, 5.4, z), 12., 9., 0.48);
    for dx in [-3., 3.] {
        window(h, Vec3::new(x + dx, 2.7, z + 4.1), 1.4, 1.8);
    }
    h.deco(
        "Neighboring front door",
        Vec3::new(x, 1.35, z + 4.1),
        Vec3::new(1.4, 2.7, 0.15),
        Teal,
    );
}

fn main_house(h: &mut HousePlan) {
    let colors = [Blue, PaleBlue, Cream, Teal, Blue];
    for (level, paint) in colors.into_iter().enumerate() {
        let y = FLOOR_Y + level as f32 * STOREY;
        // Continuous floors with a real full-height stairwell opening.
        if level == 0 {
            underground::slab_with_entrance(
                h,
                "Ground floor",
                Vec3::new(0., y - 0.12, -1.),
                Vec3::new(24.4, 0.24, 18.4),
                Wood,
            );
        } else {
            floor_with_stairwell(h, y);
        }
        if level == 4 {
            // The upper volume steps inward, leaving a covered, tree-filled terrace.
            h.block(
                "Greenhouse west wall",
                Vec3::new(-12., y + 1.8, -7.25),
                Vec3::new(0.25, 3.6, 5.5),
                paint,
                true,
            );
            for (z, depth) in [(-2.85, 3.3), (4.6, 6.8)] {
                h.block(
                    "Setback upper wall",
                    Vec3::new(-4., y + 1.8, z),
                    Vec3::new(0.25, 3.6, depth),
                    paint,
                    true,
                );
            }
            h.block(
                "Upper terrace doorway lintel",
                Vec3::new(-4., y + 3.1, 0.),
                Vec3::new(0.25, 1., 2.4),
                paint,
                true,
            );
            h.beam(
                "Roof garden diagonal support",
                Vec3::new(-11., y, 5.8),
                Vec3::new(-4., y + 3.5, 5.8),
                0.22,
                DarkWood,
            );
        } else {
            h.block(
                "West wall",
                Vec3::new(-12., y + 1.8, -1.),
                Vec3::new(0.25, 3.6, 18.),
                paint,
                true,
            );
        }
        h.block(
            "East wall",
            Vec3::new(12., y + 1.8, -1.),
            Vec3::new(0.25, 3.6, 18.),
            paint,
            true,
        );
        h.block(
            "Back wall",
            Vec3::new(0., y + 1.8, -10.),
            Vec3::new(24., 3.6, 0.25),
            paint,
            true,
        );
        let bays: &[(f32, f32)] = if level == 4 {
            &[(-0.5, 7.), (4.5, 3.), (9., 6.)]
        } else {
            &[(-9., 6.), (-4.5, 3.), (4.5, 3.), (9., 6.)]
        };
        for &(x, width) in bays {
            front_bay(h, Vec3::new(x, y, 8.), width, paint);
        }
        // Door / center bay; upper levels are open French windows.
        if level == 0 {
            for x in [-2.2, 2.2] {
                h.block(
                    "Entry side wall",
                    Vec3::new(x, y + 1.8, 8.),
                    Vec3::new(1.6, 3.6, 0.25),
                    paint,
                    true,
                );
            }
            h.block(
                "Entry lintel",
                Vec3::new(0., y + 3.18, 8.),
                Vec3::new(2.8, 0.84, 0.3),
                paint,
                true,
            );
            for x in [-1.48, 1.48] {
                h.deco(
                    "Entry jamb",
                    Vec3::new(x, y + 1.4, 8.15),
                    Vec3::new(0.16, 2.8, 0.25),
                    White,
                );
            }
        } else if level != 4 {
            front_bay(h, Vec3::new(0., y, 8.), 6., paint);
        }
        // Horizontal siding and battered white cornerboards.
        for i in 1..13 {
            let py = y + i as f32 * 0.275;
            let sides: &[(f32, f32, f32)] = if level == 4 {
                &[(-12.15, -7.25, 5.5), (-4.15, 1.75, 12.5), (12.15, -1., 18.)]
            } else {
                &[(-12.15, -1., 18.), (12.15, -1., 18.)]
            };
            for &(x, z, depth) in sides {
                if level != 4 || x != -4.15 || !(4..10).contains(&i) {
                    h.deco(
                        "Siding seam",
                        Vec3::new(x, py, z),
                        Vec3::new(0.025, 0.025, depth),
                        DarkWood,
                    );
                }
            }
            // Front seams stop before window openings.
            if !(4..10).contains(&i) {
                h.deco(
                    "Front clapboard seam",
                    Vec3::new(if level == 4 { 4. } else { 0. }, py, 8.145),
                    Vec3::new(if level == 4 { 16. } else { 24. }, 0.026, 0.018),
                    Teal,
                );
            }
        }
        let corners = if level == 4 {
            [
                (-12.17, -10.1),
                (-4.17, 8.12),
                (12.17, -10.1),
                (12.17, 8.12),
            ]
        } else {
            [
                (-12.17, -10.1),
                (-12.17, 8.12),
                (12.17, -10.1),
                (12.17, 8.12),
            ]
        };
        for (x, z) in corners {
            h.deco(
                "Cornerboard",
                Vec3::new(x, y + 1.8, z),
                Vec3::new(0.23, 3.6, 0.23),
                White,
            );
        }
        h.deco(
            "Storey cornice",
            Vec3::new(if level == 4 { 4. } else { 0. }, y + 3.52, 8.26),
            Vec3::new(if level == 4 { 16.7 } else { 24.7 }, 0.16, 0.45),
            White,
        );
        // Wallpaper, rugs and the navigable landing form a distinct interior on each level.
        h.deco(
            "Hall runner",
            Vec3::new(2., y + 0.015, 0.),
            Vec3::new(3.1, 0.025, 13.5),
            if level % 2 == 0 { Red } else { Teal },
        );
        h.room(
            match level {
                0 => "Floor 1 · living room",
                1 => "Floor 2 · workshop & bedroom",
                2 => "Floor 3 · gallery & child's room",
                3 => "Floor 4 · trolley station",
                _ => "Floor 5 · machinery & greenhouse",
            },
            Vec3::new(0., y + 1.5, -1.),
            Vec3::new(24., 3.1, 18.),
        );
        stairs(h, y, level);
    }
    floor_with_stairwell(h, FLOOR_Y + 5. * STOREY);
    h.room(
        "Floor 6 · roof garden & windmill",
        Vec3::new(0., 19.4, -1.),
        Vec3::new(29., 4., 24.),
    );
    // Porches and their deliberately leaning supports.
    h.block(
        "Covered porch deck",
        Vec3::new(0., 0.12, 9.5),
        Vec3::new(10., 0.24, 3.),
        Wood,
        true,
    );
    for i in 0..4 {
        h.block(
            "Porch step",
            Vec3::new(0., 0.03 * (4 - i) as f32, 11.05 + i as f32 * 0.3),
            Vec3::new(3.2, 0.06 * (4 - i) as f32, 0.3),
            Wood,
            true,
        );
    }
    for x in [-4.6, -1.8, 1.8, 4.6] {
        h.beam(
            "Porch support",
            Vec3::new(x, 0.2, 10.65),
            Vec3::new(x + 0.15, 3.1, 10.5),
            0.18,
            White,
        );
    }
    h.rotated(
        "Porch roof",
        Vec3::new(0., 3.2, 9.65),
        Vec3::new(10.8, 0.25, 3.9),
        Quat::from_rotation_x(0.08),
        Roof,
    );
    // Front balcony on the second floor.
    h.block(
        "Front balcony",
        Vec3::new(-7., 3.64, 9.2),
        Vec3::new(7.5, 0.22, 2.4),
        Wood,
        true,
    );
    railing(h, Vec3::new(-7., 4.2, 10.4), 7.6);
    for x in [-10.6, -3.4] {
        h.beam(
            "Balcony brace",
            Vec3::new(x, 1.8, 8.1),
            Vec3::new(x, 3.5, 10.2),
            0.17,
            DarkWood,
        );
    }
}
fn floor_with_stairwell(h: &mut HousePlan, y: f32) {
    for (center, size) in [
        (Vec3::new(-3.55, y - 0.12, -1.), Vec3::new(16.9, 0.24, 18.4)),
        (Vec3::new(11.65, y - 0.12, -1.), Vec3::new(1.1, 0.24, 18.4)),
        (Vec3::new(8., y - 0.12, -8.2), Vec3::new(6.2, 0.24, 4.)),
        (Vec3::new(8., y - 0.12, 6.4), Vec3::new(6.2, 0.24, 3.6)),
    ] {
        h.block("Floor around stairwell", center, size, Wood, true);
    }
}
fn stairs(h: &mut HousePlan, base: f32, level: usize) {
    let ascending_back = level.is_multiple_of(2);
    let x = if ascending_back { 6.5 } else { 9.6 };
    for i in 0..18 {
        let top = (i + 1) as f32 * 0.2;
        let z = if ascending_back {
            4.44 - (i as f32 + 0.5) * 0.6
        } else {
            -6.36 + (i as f32 + 0.5) * 0.6
        };
        h.block(
            "Stair tread",
            Vec3::new(x, base + top - 0.1, z),
            Vec3::new(2.6, 0.2, 0.6),
            DarkWood,
            true,
        );
        h.deco(
            "Stair nosing",
            Vec3::new(
                x,
                base + top + 0.005,
                z + if ascending_back { 0.27 } else { -0.27 },
            ),
            Vec3::new(2.6, 0.04, 0.06),
            Cream,
        );
    }
    let (start, end) = if ascending_back {
        (4.44, -6.36)
    } else {
        (-6.36, 4.44)
    };
    h.beam(
        "Stair handrail",
        Vec3::new(x - 1.4, base + 0.95, start),
        Vec3::new(x - 1.4, base + 4.55, end),
        0.1,
        White,
    );
}
fn front_bay(h: &mut HousePlan, p: Vec3, width: f32, paint: crate::world::Paint) {
    let window_width = 1.65;
    let side = (width - window_width) * 0.5;
    for sign in [-1., 1.] {
        h.block(
            "Window pier",
            p + Vec3::new(sign * (window_width + side) * 0.5, 1.8, 0.),
            Vec3::new(side, 3.6, 0.25),
            paint,
            true,
        );
    }
    h.block(
        "Window sill wall",
        p + Vec3::new(0., 0.5, 0.),
        Vec3::new(window_width, 1., 0.25),
        paint,
        true,
    );
    h.block(
        "Window header",
        p + Vec3::new(0., 3.16, 0.),
        Vec3::new(window_width, 0.88, 0.25),
        paint,
        true,
    );
    window(h, p + Vec3::new(0., 1.86, 0.17), window_width, 1.72);
}
fn window(h: &mut HousePlan, p: Vec3, w: f32, height: f32) {
    // Openable-looking windows are intentionally unglazed routes in this blockout.
    for x in [-w * 0.5, w * 0.5] {
        h.deco(
            "Window frame",
            p + Vec3::new(x, 0., 0.),
            Vec3::new(0.12, height + 0.24, 0.18),
            White,
        );
    }
    for y in [-height * 0.5, height * 0.5] {
        h.deco(
            "Window frame",
            p + Vec3::new(0., y, 0.),
            Vec3::new(w + 0.24, 0.12, 0.18),
            White,
        );
    }
    h.deco("Window mullion", p, Vec3::new(0.06, height, 0.1), Cream);
    h.deco("Window crossbar", p, Vec3::new(w, 0.06, 0.1), Cream);
    h.deco(
        "Window drip sill",
        p + Vec3::new(0., -height * 0.5 - 0.1, 0.07),
        Vec3::new(w + 0.4, 0.1, 0.4),
        White,
    );
}
fn roof(h: &mut HousePlan, base: Vec3, w: f32, d: f32, angle: f32) {
    let half = w * 0.5;
    let rise = half * angle.tan();
    for sign in [-1., 1.] {
        h.rotated(
            "Sloping shingle roof",
            base + Vec3::new(sign * half * 0.5, rise * 0.5, 0.),
            Vec3::new(half / angle.cos() + 0.4, 0.22, d),
            Quat::from_rotation_z(-sign * angle),
            Roof,
        );
    }
    h.beam(
        "Roof ridge",
        base + Vec3::new(0., rise, -d * 0.5),
        base + Vec3::new(0., rise, d * 0.5),
        0.16,
        DarkWood,
    );
}
fn railing(h: &mut HousePlan, p: Vec3, w: f32) {
    h.block(
        "Balcony rail",
        p + Vec3::Y * 0.4,
        Vec3::new(w, 0.12, 0.12),
        White,
        true,
    );
    for i in 0..=(w / 0.55) as usize {
        h.deco(
            "Baluster",
            p + Vec3::new(-w * 0.5 + i as f32 * 0.55, 0., 0.),
            Vec3::new(0.08, 0.8, 0.08),
            Cream,
        );
    }
}

fn interiors(h: &mut HousePlan) {
    let y = FLOOR_Y;
    // The basement wall is a real obstacle with a separate dynamic locked door.
    h.block(
        "Basement enclosure west",
        Vec3::new(-10., y + 1.5, -5.),
        Vec3::new(1.6, 3., 0.3),
        Wallpaper,
        true,
    );
    h.block(
        "Basement enclosure east",
        Vec3::new(-5.5, y + 1.5, -5.),
        Vec3::new(2.6, 3., 0.3),
        Wallpaper,
        true,
    );
    h.block(
        "Basement lintel",
        BASEMENT_DOOR + Vec3::Y * 1.35,
        Vec3::new(2.4, 0.35, 0.4),
        DarkWood,
        true,
    );
    for i in 0..9 {
        h.deco(
            "Basement hazard stripe",
            BASEMENT_DOOR + Vec3::new(-1.2 + i as f32 * 0.3, 1.6, 0.18),
            Vec3::new(0.15, 0.12, 0.08),
            Yellow,
        );
    }
    h.room(
        "Basement door · boards + keycard",
        Vec3::new(-8., 1., -3.7),
        Vec3::new(5., 3., 3.),
    );
    // Living room: tilted TV, sofa, patterned rug, phone, paintings and floor lamp.
    sofa(h, Vec3::new(-7.5, y, 4.5), Teal);
    h.block(
        "Television stand",
        Vec3::new(-8., y + 0.38, 0.8),
        Vec3::new(2.4, 0.75, 0.7),
        DarkWood,
        true,
    );
    h.rotated(
        "CRT television",
        Vec3::new(-8., y + 1.2, 0.8),
        Vec3::new(1.4, 1., 0.8),
        Quat::from_rotation_z(-0.06),
        DarkWood,
    );
    h.deco(
        "CRT screen",
        Vec3::new(-8., y + 1.23, 1.23),
        Vec3::new(1.05, 0.68, 0.04),
        Glass,
    );
    for dx in [-0.4, 0.4] {
        h.beam(
            "TV antenna",
            Vec3::new(-8., y + 1.75, 0.8),
            Vec3::new(-8. + dx, y + 2.4, 0.8),
            0.025,
            Metal,
        );
    }
    h.deco(
        "Living room rug",
        Vec3::new(-7.2, y + 0.02, 2.9),
        Vec3::new(5., 0.025, 3.5),
        Red,
    );
    for i in 0..6 {
        h.deco(
            "Carpet stripes",
            Vec3::new(-9.4 + i as f32 * 0.85, y + 0.04, 2.9),
            Vec3::new(0.08, 0.02, 3.2),
            Cream,
        );
    }
    table(h, Vec3::new(-4.5, y, 5.6), Vec3::new(1.3, 0.8, 0.7));
    h.deco(
        "Telephone",
        Vec3::new(-4.5, y + 0.9, 5.6),
        Vec3::new(0.42, 0.2, 0.3),
        Black,
    );
    lamp(h, Vec3::new(-10.8, y, 3.));
    h.deco(
        "Porch doorbell",
        Vec3::new(1.65, y + 1.35, 8.3),
        Vec3::new(0.16, 0.3, 0.09),
        Metal,
    );
    // Kitchen: fridge/freezer and old-fashioned cooker, cabinets, sink and red kettle.
    h.sphere(
        "Lamp shade",
        Vec3::new(-5.2, y + 2.95, -8.3),
        Vec3::new(0.3, 0.12, 0.3),
        Cream,
    );
    h.room(
        "Kitchen · locked freezer",
        Vec3::new(-6., y + 1.4, -8.),
        Vec3::new(10., 3., 4.),
    );
    h.block(
        "Refrigerator",
        Vec3::new(FREEZER.x, y + 1.25, FREEZER.z),
        Vec3::new(1.6, 2.5, 1.3),
        White,
        true,
    );
    h.deco(
        "Freezer door",
        Vec3::new(FREEZER.x, y + 1.92, FREEZER.z + 0.68),
        Vec3::new(1.43, 0.88, 0.08),
        PaleBlue,
    );
    h.deco(
        "Fridge handle",
        Vec3::new(FREEZER.x + 0.5, y + 1.5, FREEZER.z + 0.76),
        Vec3::new(0.07, 0.7, 0.1),
        Metal,
    );
    for i in 0..4 {
        let x = -6.6 + i as f32 * 1.4;
        h.block(
            "Kitchen cabinet",
            Vec3::new(x, y + 0.48, -9.),
            Vec3::new(1.3, 0.96, 1.4),
            Cream,
            true,
        );
        h.deco(
            "Kitchen counter",
            Vec3::new(x, y + 1., -9.),
            Vec3::new(1.4, 0.12, 1.5),
            White,
        );
        h.deco(
            "Drawer pull",
            Vec3::new(x, y + 0.75, -8.26),
            Vec3::new(0.3, 0.05, 0.08),
            Metal,
        );
    }
    h.deco(
        "Sink",
        Vec3::new(-5.2, y + 1.08, -9.),
        Vec3::new(0.9, 0.04, 0.8),
        Metal,
    );
    for x in [-5.65, -4.75] {
        h.deco(
            "Sink rim",
            Vec3::new(x, y + 1.12, -9.),
            Vec3::new(0.035, 0.06, 0.8),
            Metal,
        );
    }
    h.pipe(
        Vec3::new(-5.2, y + 1.6, -9.4),
        Vec3::new(-5.2, y + 1.6, -9.05),
        0.045,
        Metal,
    );
    h.pipe(
        Vec3::new(-5.2, y + 1.1, -9.4),
        Vec3::new(-5.2, y + 1.6, -9.4),
        0.055,
        Metal,
    );
    h.sphere(
        "Red kettle",
        Vec3::new(-2.4, y + 1.25, -9.),
        Vec3::splat(0.22),
        Red,
    );
    // First upper floor: workshop, bedroom and bathroom.
    let y = y + STOREY;
    table(h, Vec3::new(-8., y, -6.9), Vec3::new(3.4, 0.95, 1.3));
    shelf(h, Vec3::new(-11., y, -6.5));
    h.room(
        "Workshop · crowbar",
        Vec3::new(-8., y + 1., -6.),
        Vec3::new(7., 3., 6.),
    );
    bed(h, Vec3::new(-7.5, y, 3.5));
    lamp(h, Vec3::new(-10.8, y, 3.));
    h.room(
        "Peterson's bedroom",
        Vec3::new(-7., y + 1., 3.),
        Vec3::new(8., 3., 7.),
    );
    h.block(
        "Bath tub",
        Vec3::new(3.7, y + 0.4, -7.9),
        Vec3::new(2.5, 0.8, 1.1),
        White,
        true,
    );
    h.pipe(
        Vec3::new(4.8, y + 0.5, -8.1),
        Vec3::new(4.8, y + 2.7, -8.1),
        0.06,
        Metal,
    );
    h.pipe(
        Vec3::new(4.8, y + 2.7, -8.1),
        Vec3::new(4.2, y + 2.7, -8.1),
        0.06,
        Metal,
    );
    h.deco(
        "Shower valve",
        Vec3::new(4.8, y + 1.5, -7.95),
        Vec3::new(0.18, 0.12, 0.14),
        Metal,
    );
    h.sphere(
        "Lamp shade",
        Vec3::new(3.4, y + 2.95, -7.4),
        Vec3::new(0.3, 0.12, 0.3),
        Cream,
    );
    h.room(
        "Bathroom · pipes & tub",
        Vec3::new(3., y + 1., -7.6),
        Vec3::new(6., 3., 4.),
    );
    // Second upper floor: gallery, child room, wardrobe and dollhouse oddities.
    let y = y + STOREY;
    shelf(h, Vec3::new(-10.8, y, -7.5));
    table(h, Vec3::new(-8., y, -7.5), Vec3::new(1.8, 0.85, 0.9));
    h.block(
        "Child dresser",
        Vec3::new(-9., y + 0.6, 3.5),
        Vec3::new(2., 1.2, 0.9),
        PaleBlue,
        true,
    );
    h.deco(
        "Dresser drawer",
        Vec3::new(-9., y + 0.85, 4.04),
        Vec3::new(0.3, 0.05, 0.08),
        Metal,
    );
    h.deco(
        "Toy skateboard",
        Vec3::new(-8.5, y + 0.15, 5.),
        Vec3::new(0.35, 0.12, 1.),
        Red,
    );
    h.room(
        "Child's room · picture clues",
        Vec3::new(-8., y + 1., 4.),
        Vec3::new(8., 3., 7.),
    );
    lamp(h, Vec3::new(-10.8, y, 3.));
    for i in 0..5 {
        painting(h, Vec3::new(-9.5 + i as f32 * 3.6, y + 2., -9.8), i);
    }
    h.block(
        "Wardrobe",
        Vec3::new(-4.5, y + 1.25, -7.5),
        Vec3::new(2., 2.5, 1.2),
        DarkWood,
        true,
    );
    h.deco(
        "Wardrobe doors",
        Vec3::new(-4.5, y + 1.25, -6.85),
        Vec3::new(1.8, 2.3, 0.1),
        Wood,
    );
    h.room(
        "Gallery · crooked paintings",
        Vec3::new(-2., y + 1., -7.),
        Vec3::new(10., 3., 5.),
    );
    // Station floor: rail control, an ominous mannequin and suspended lamps.
    let y = y + STOREY;
    h.block(
        "Trolley control console",
        TRAIN_SWITCH - Vec3::Y * 0.65,
        Vec3::new(2., 1., 1.),
        Teal,
        true,
    );
    h.room(
        "Station & control room",
        Vec3::new(-7., y + 1., 3.),
        Vec3::new(9., 3., 9.),
    );
    for i in 0..3 {
        h.sphere(
            "Control indicator",
            TRAIN_SWITCH + Vec3::new(-0.5 + i as f32 * 0.5, -0.1, 0.54),
            Vec3::splat(0.09),
            Glow,
        );
    }
    for z in [-7., 0., 6.] {
        lamp(h, Vec3::new(-10.5, y, z));
    }
    // Machinery floor: generator, giant magnetic coil, greenhouse seedlings.
    let y = y + STOREY;
    h.block(
        "Generator body",
        GENERATOR - Vec3::Y * 0.4,
        Vec3::new(2.2, 1.5, 1.5),
        Red,
        true,
    );
    for i in 0..5 {
        h.deco(
            "Generator grille",
            GENERATOR + Vec3::new(-0.7 + i as f32 * 0.35, -0.4, 0.8),
            Vec3::new(0.1, 0.85, 0.04),
            Black,
        );
    }
    h.room(
        "Generator room · freezer power",
        Vec3::new(-8., y + 1., -7.),
        Vec3::new(7., 3., 5.),
    );
    h.pipe(
        Vec3::new(-4., y + 0.6, -7.),
        Vec3::new(-4., y + 2.6, -7.),
        0.6,
        Metal,
    );
    for i in 0..8 {
        h.deco(
            "Magnetic coil rings",
            Vec3::new(-4., y + 0.65 + i as f32 * 0.24, -7.),
            Vec3::new(1.45, 0.08, 1.45),
            Orange,
        );
    }
    for x in [-9., -6., -3.] {
        h.block(
            "Greenhouse planter",
            Vec3::new(x, y + 0.3, 4.),
            Vec3::new(2., 0.6, 2.),
            DarkWood,
            true,
        );
        tree(h, Vec3::new(x, y + 0.6, 4.), 0.35);
    }
    h.room(
        "Greenhouse · indoor trees",
        Vec3::new(-6., y + 1., 4.),
        Vec3::new(12., 3., 7.),
    );
    // Upper roof garden, aerials, chairs and a large windmill switch.
    for (x, z, s) in [(-8., -5., 0.7), (-8., 3., 0.8), (-3., 1., 0.65)] {
        tree(h, Vec3::new(x, 18.12, z), s);
    }
    h.block(
        "Windmill maintenance box",
        WINDMILL_SWITCH - Vec3::Y * 0.4,
        Vec3::new(1.2, 0.9, 0.7),
        Teal,
        true,
    );
    for x in [-10., -5., 0., 5., 10.] {
        h.pipe(Vec3::new(x, 18.2, -9.), Vec3::new(x, 20., -9.), 0.05, Metal);
    }
    // Readable, genuinely different small props throughout every floor.
    for level in 0..5 {
        let y = FLOOR_Y + level as f32 * STOREY;
        for j in 0..3 {
            let x = -1.8 - j as f32 * 2.1;
            h.deco(
                "Wallpaper motif",
                Vec3::new(x, y + 1.6, -9.83),
                Vec3::new(0.14, 0.8, 0.03),
                Orange,
            );
        }
        table(h, Vec3::new(3.6, y, 5.7), Vec3::new(1.7, 0.8, 1.1));
        h.sphere(
            "Apple",
            Vec3::new(3.7, y + 0.97, 5.7),
            Vec3::splat(0.16),
            Red,
        );
        h.deco(
            "Book",
            Vec3::new(3.1, y + 0.92, 5.7),
            Vec3::new(0.4, 0.12, 0.6),
            Teal,
        );
    }
}

fn impossible_additions(h: &mut HousePlan) {
    // Projecting rooms, mismatched roofs and timber stilts break the box silhouette.
    for (center, size, paint) in [
        (Vec3::new(-14., 8.9, 0.), Vec3::new(5., 3.6, 8.), Yellow),
        (Vec3::new(13.8, 13., -2.), Vec3::new(4., 4., 8.), Red),
        (Vec3::new(-10., 19.8, -6.), Vec3::new(5., 3.6, 5.), PaleBlue),
        (Vec3::new(15., 20., 0.), Vec3::new(6., 4., 6.), Teal),
    ] {
        h.block("Cantilevered room", center, size, paint, true);
        roof(
            h,
            center + Vec3::Y * size.y * 0.5,
            size.x + 0.8,
            size.z + 0.8,
            0.5,
        );
        window(h, center + Vec3::new(0., 0., size.z * 0.5 + 0.12), 1.8, 1.9);
        for x in [-size.x * 0.4, size.x * 0.4] {
            h.beam(
                "Impossible timber stilts",
                Vec3::new(center.x + x, 0., center.z),
                Vec3::new(center.x + x + 0.6, center.y - size.y * 0.5, center.z),
                0.27,
                DarkWood,
            );
        }
    }
    // High, narrow rear tower and its crooked cap.
    // Watchtower is a traversable room, authored below, rather than a solid block.
    roof(h, Vec3::new(2., 24., -6.), 6.5, 5.5, 0.65);
    window(h, Vec3::new(2., 22.2, -3.86), 1.8, 2.2);
    h.pipe(
        Vec3::new(11., 1., -10.8),
        Vec3::new(11., 22., -10.8),
        0.25,
        Orange,
    );
    h.pipe(
        Vec3::new(11., 22., -10.8),
        Vec3::new(6., 22., -10.8),
        0.25,
        Orange,
    );
    h.pipe(
        Vec3::new(-12.8, 1., 6.),
        Vec3::new(-12.8, 12.8, 6.),
        0.28,
        Metal,
    );
    h.pipe(
        Vec3::new(-12.8, 12.8, 6.),
        Vec3::new(-17., 12.8, 6.),
        0.28,
        Metal,
    );
    // Four fear-room doors are not all made into fake functional minigames.
    // The hanging shed is an architectural landmark, reserved for Fear Darkness.
    let shed = Vec3::new(-20., 14., -7.);
    h.block(
        "Suspended fear-room shed",
        shed,
        Vec3::new(4., 3.5, 4.),
        DarkWood,
        true,
    );
    roof(h, shed + Vec3::Y * 1.75, 4.7, 4.7, 0.55);
    window(h, shed + Vec3::new(0., 0.1, 2.1), 1.2, 1.4);
    for x in [-21.6, -18.4] {
        h.beam(
            "Hanging shed cables",
            Vec3::new(x, 15.8, -7.),
            Vec3::new(-11., 22., -7.),
            0.055,
            Metal,
        );
    }
    // Trolley rails: a closed, elevated rectangular route with sleepers and support trestles.
    for z in [-12., 11.6] {
        for x in [-16.5, 16.5] {
            h.beam(
                "Rail trestle",
                Vec3::new(x, 0., z),
                Vec3::new(x, 11.6, z),
                0.25,
                DarkWood,
            );
            h.beam(
                "Rail diagonal brace",
                Vec3::new(x - 1.2, 0., z),
                Vec3::new(x + 1.2, 11.6, z),
                0.14,
                DarkWood,
            );
        }
    }
    for offset in [-0.55, 0.55] {
        for z in [-12. + offset, 11.6 + offset] {
            h.deco(
                "Train rail",
                Vec3::new(0., 11.55, z),
                Vec3::new(33., 0.12, 0.1),
                Metal,
            );
        }
        for x in [-16.5 + offset, 16.5 + offset] {
            h.deco(
                "Train rail",
                Vec3::new(x, 11.55, -0.2),
                Vec3::new(0.1, 0.12, 23.2),
                Metal,
            );
        }
    }
    for i in 0..57 {
        let x = -16.8 + i as f32 * 0.6;
        for z in [-12., 11.6] {
            h.deco(
                "Railway sleeper",
                Vec3::new(x, 11.42, z),
                Vec3::new(0.2, 0.13, 1.6),
                DarkWood,
            );
        }
    }
    for i in 0..39 {
        for x in [-16.5, 16.5] {
            h.deco(
                "Railway sleeper",
                Vec3::new(x, 11.42, -11.8 + i as f32 * 0.6),
                Vec3::new(1.6, 0.13, 0.2),
                DarkWood,
            );
        }
    }
    h.block(
        "Station access balcony",
        Vec3::new(-8., 10.85, 9.7),
        Vec3::new(5., 0.25, 3.8),
        Wood,
        true,
    );
    // Giant clock-like rooftop windmill. The blades are spawned as an animated hierarchy.
    h.beam(
        "Windmill mast",
        Vec3::new(-3.7, 18., -5.),
        Vec3::new(-3.7, 28., -5.),
        0.42,
        DarkWood,
    );
    h.beam(
        "Windmill brace",
        Vec3::new(-6., 18., -5.),
        Vec3::new(-3.7, 27.7, -5.),
        0.22,
        DarkWood,
    );
    h.pipe(
        Vec3::new(-3.7, 28., -6.),
        Vec3::new(-3.7, 28., -3.7),
        0.22,
        Metal,
    );
    // Water tower / tank and red exterior ladder.
    h.pipe(
        Vec3::new(14.6, 17., -8.),
        Vec3::new(14.6, 20., -8.),
        1.7,
        Metal,
    );
    for x in [13.4, 15.8] {
        h.beam(
            "Water tank supports",
            Vec3::new(x, 0., -8.),
            Vec3::new(x, 17., -8.),
            0.22,
            DarkWood,
        );
    }
    for x in [12.5, 13.6] {
        h.beam(
            "Exterior ladder",
            Vec3::new(x, 0., 3.),
            Vec3::new(x, 15., 3.),
            0.09,
            Red,
        );
    }
    for i in 0..38 {
        h.deco(
            "Ladder rung",
            Vec3::new(13.05, i as f32 * 0.4, 3.),
            Vec3::new(1.1, 0.07, 0.1),
            Red,
        );
    }
}

fn table(h: &mut HousePlan, p: Vec3, size: Vec3) {
    h.block(
        "Tabletop",
        p + Vec3::Y * size.y,
        Vec3::new(size.x, 0.14, size.z),
        Wood,
        true,
    );
    for x in [-1., 1.] {
        for z in [-1., 1.] {
            h.block(
                "Table leg",
                p + Vec3::new(
                    x * (size.x * 0.5 - 0.1),
                    size.y * 0.5,
                    z * (size.z * 0.5 - 0.1),
                ),
                Vec3::new(0.12, size.y, 0.12),
                DarkWood,
                true,
            );
        }
    }
}
fn sofa(h: &mut HousePlan, p: Vec3, paint: crate::world::Paint) {
    h.block(
        "Sofa seat",
        p + Vec3::Y * 0.45,
        Vec3::new(3.2, 0.6, 1.3),
        paint,
        true,
    );
    h.block(
        "Sofa back",
        p + Vec3::new(0., 1., 0.5),
        Vec3::new(3.2, 1.1, 0.3),
        paint,
        true,
    );
    for x in [-1.55, 1.55] {
        h.block(
            "Sofa arm",
            p + Vec3::new(x, 0.75, 0.),
            Vec3::new(0.3, 0.8, 1.4),
            paint,
            true,
        );
    }
    for x in [-0.95, 0., 0.95] {
        h.deco(
            "Sofa cushion",
            p + Vec3::new(x, 0.8, -0.08),
            Vec3::new(0.85, 0.2, 0.9),
            PaleBlue,
        );
    }
}
fn bed(h: &mut HousePlan, p: Vec3) {
    h.block(
        "Bed frame",
        p + Vec3::Y * 0.35,
        Vec3::new(2.3, 0.7, 3.5),
        DarkWood,
        true,
    );
    h.deco(
        "Mattress",
        p + Vec3::Y * 0.8,
        Vec3::new(2.2, 0.3, 3.4),
        White,
    );
    h.deco(
        "Quilt",
        p + Vec3::new(0., 0.99, 0.5),
        Vec3::new(2.2, 0.1, 2.3),
        Teal,
    );
    h.deco(
        "Pillow",
        p + Vec3::new(0., 1., -1.15),
        Vec3::new(1.5, 0.24, 0.7),
        Cream,
    );
    h.block(
        "Bed headboard",
        p + Vec3::new(0., 0.8, -1.8),
        Vec3::new(2.5, 1.6, 0.16),
        Wood,
        true,
    );
}
fn shelf(h: &mut HousePlan, p: Vec3) {
    for x in [-0.8, 0.8] {
        h.block(
            "Bookshelf upright",
            p + Vec3::new(x, 1.3, 0.),
            Vec3::new(0.13, 2.6, 0.6),
            DarkWood,
            true,
        );
    }
    for i in 0..4 {
        let y = 0.15 + i as f32 * 0.75;
        h.deco("Bookshelf", p + Vec3::Y * y, Vec3::new(1.7, 0.1, 0.6), Wood);
        for j in 0..5 {
            h.deco(
                "Books",
                p + Vec3::new(-0.6 + j as f32 * 0.27, y + 0.25, 0.),
                Vec3::new(0.17, 0.4 + (j % 2) as f32 * 0.1, 0.4),
                [Red, Teal, Cream][j % 3],
            );
        }
    }
}
fn lamp(h: &mut HousePlan, p: Vec3) {
    h.pipe(p, p + Vec3::Y * 2.1, 0.04, Metal);
    h.sphere(
        "Lamp shade",
        p + Vec3::Y * 2.05,
        Vec3::new(0.45, 0.35, 0.45),
        Cream,
    );
    h.deco(
        "Lamp base",
        p + Vec3::Y * 0.04,
        Vec3::new(0.5, 0.08, 0.5),
        Black,
    );
}
fn painting(h: &mut HousePlan, p: Vec3, i: usize) {
    h.rotated(
        "Crooked painting frame",
        p,
        Vec3::new(1.4, 1.5, 0.1),
        Quat::from_rotation_z((i as f32 - 2.) * 0.06),
        DarkWood,
    );
    h.deco(
        "Painting canvas",
        p + Vec3::Z * 0.07,
        Vec3::new(1.15, 1.24, 0.03),
        Cream,
    );
    h.sphere(
        "Painted apple",
        p + Vec3::Z * 0.1,
        Vec3::new(0.4, 0.4, 0.02),
        [Red, Teal, Orange][i % 3],
    );
}
fn tree(h: &mut HousePlan, p: Vec3, s: f32) {
    h.pipe(p, p + Vec3::Y * (4.2 * s), 0.3 * s, Trunk);
    for (dx, dy, dz, r) in [
        (0., 4., 0., 2.2),
        (-1.1, 3.3, 0.4, 1.7),
        (1., 3.8, -0.3, 1.7),
        (0.1, 5.2, 0., 1.5),
    ] {
        h.sphere(
            "Angular tree canopy",
            p + Vec3::new(dx, dy, dz) * s,
            Vec3::splat(r * s),
            Leaf,
        );
    }
}
fn navigation(h: &mut HousePlan) {
    let outside = h.node(Vec3::new(0., 0., 12.5));
    let front = h.node(Vec3::new(0., FLOOR_Y, 6.));
    h.link(outside, front);
    let living = h.node(Vec3::new(-4., FLOOR_Y, 3.));
    h.link(front, living);
    let kitchen = h.node(Vec3::new(-3., FLOOR_Y, -6.));
    h.link(living, kitchen);
    let yard_west = h.node(Vec3::new(-15., 0., 12.));
    h.link(outside, yard_west);
    let yard_east = h.node(Vec3::new(17., 0., 12.));
    h.link(outside, yard_east);
    h.patrol = vec![living, kitchen, front, yard_east, front, yard_west];
    let mut last_landing = front;
    for level in 0usize..5 {
        let base = FLOOR_Y + level as f32 * STOREY;
        let ascending_back = level.is_multiple_of(2);
        let x = if ascending_back { 6.5 } else { 9.6 };
        let start_z = if ascending_back { 5.2 } else { -7.2 };
        let end_z = if ascending_back { -7.2 } else { 5.2 };
        let corridor = h.node(Vec3::new(4.0, base, start_z));
        h.link(last_landing, corridor);
        let bottom = h.node(Vec3::new(x, base, start_z));
        h.link(corridor, bottom);
        let mut previous = bottom;
        for i in 0..18 {
            let z = if ascending_back {
                4.44 - (i as f32 + 0.5) * 0.6
            } else {
                -6.36 + (i as f32 + 0.5) * 0.6
            };
            let step = h.node(Vec3::new(x, base + (i + 1) as f32 * 0.2, z));
            h.link(previous, step);
            previous = step;
        }
        let top = h.node(Vec3::new(x, base + STOREY, end_z));
        h.link(previous, top);
        let landing = h.node(Vec3::new(4.0, base + STOREY, end_z));
        h.link(top, landing);
        let hall_front = h.node(Vec3::new(2.5, base + STOREY, 5.2));
        h.link(landing, hall_front);
        let hall_rear = h.node(Vec3::new(2.5, base + STOREY, -7.2));
        h.link(hall_front, hall_rear);
        let room_front = h.node(Vec3::new(
            if level == 3 { -2. } else { -4. },
            base + STOREY,
            3.,
        ));
        h.link(hall_front, room_front);
        let room_rear = h.node(Vec3::new(-3., base + STOREY, -6.));
        h.link(hall_rear, room_rear);
        let threshold = h.node(Vec3::new(-8., base + STOREY, -3.5));
        let locked_room = h.node(Vec3::new(-8., base + STOREY, -7.0));
        h.link(room_front, threshold);
        h.link(threshold, locked_room);
        last_landing = landing;
    }
}

/// Each key room has a proper shell: its doorway cannot be bypassed by walking around a panel.
fn mystery_rooms(h: &mut HousePlan) {
    for level in [2, 4] {
        let y = FLOOR_Y + level as f32 * STOREY;
        h.block(
            "Locked room east wall",
            Vec3::new(-5., y + 1.8, -7.25),
            Vec3::new(0.25, 3.6, 5.5),
            Wallpaper,
            true,
        );
        for (x, width) in [(-10.65, 2.7), (-5.9, 1.8)] {
            h.block(
                "Mystery doorway wall",
                Vec3::new(x, y + 1.8, -4.5),
                Vec3::new(width, 3.6, 0.25),
                if level == 2 { Blue } else { Teal },
                true,
            );
        }
        h.block(
            "Mystery door lintel",
            Vec3::new(-8., y + 3.1, -4.5),
            Vec3::new(2.6, 1., 0.3),
            DarkWood,
            true,
        );
        h.room(
            if level == 2 {
                "Sealed archive · blue key"
            } else {
                "Locked greenhouse · green key"
            },
            Vec3::new(-8.5, y + 1.5, -7.2),
            Vec3::new(6.5, 3., 5.5),
        );
    }
    let base = FLOOR_Y + 5. * STOREY;
    // Two-storey tower with an upper machinery gallery and ladder shortcut.
    for x in [-0.5, 4.5] {
        h.block(
            "Tower wall",
            Vec3::new(x, base + 3., -6.),
            Vec3::new(0.25, 6., 4.2),
            Blue,
            true,
        );
    }
    h.block(
        "Tower rear wall",
        Vec3::new(2., base + 3., -8.),
        Vec3::new(5., 6., 0.25),
        Teal,
        true,
    );
    for x in [0.15, 3.85] {
        h.block(
            "Tower front pier",
            Vec3::new(x, base + 3., -4.),
            Vec3::new(1.3, 6., 0.25),
            Blue,
            true,
        );
    }
    h.block(
        "Tower doorway header",
        Vec3::new(2., base + 4.35, -4.),
        Vec3::new(2.4, 3.3, 0.25),
        Blue,
        true,
    );
    h.block(
        "Tower upper machinery floor",
        Vec3::new(2., base + 3.48, -6.),
        Vec3::new(4.8, 0.24, 3.8),
        DarkWood,
        true,
    );
    for x in [0.4, 1.3] {
        h.beam(
            "Tower climbing ladder",
            Vec3::new(x, base, -6.6),
            Vec3::new(x, base + 5., -6.6),
            0.07,
            Yellow,
        );
    }
    for i in 0..13 {
        h.deco(
            "Tower ladder rung",
            Vec3::new(0.85, base + i as f32 * 0.4, -6.6),
            Vec3::new(1., 0.07, 0.1),
            Yellow,
        );
    }
    h.room(
        "Red tower · listening & dynamo gallery",
        Vec3::new(2., base + 3., -6.),
        Vec3::new(5., 6., 4.),
    );
    // Mechanical ears, resonating brass horns, clock faces and a power cable to the generator.
    for side in [-1., 1.] {
        h.pipe(
            Vec3::new(2. + side * 0.9, 22.2, -6.),
            Vec3::new(2. + side * 0.9, 23.3, -6.),
            0.14,
            Metal,
        );
        h.pieces.push(crate::world::Piece {
            name: "Acoustic listening horn",
            transform: Transform::from_xyz(2. + side * 0.9, 23.3, -6.)
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::new(0.4, 0.5, 0.4)),
            paint: Yellow,
            shape: crate::world::Shape::Cone,
        });
        h.sphere(
            "Horn dark opening",
            Vec3::new(2. + side * 0.9, 23.3, -5.74),
            Vec3::new(0.32, 0.32, 0.018),
            Black,
        );
    }
    h.pipe(
        Vec3::new(-3.7, 28., -5.),
        Vec3::new(2., 24., -5.),
        0.12,
        Metal,
    );
    h.pipe(Vec3::new(2., 24., -5.), WINDMILL_SWITCH, 0.12, Metal);
    h.beam(
        "Generator cable",
        WINDMILL_SWITCH,
        Vec3::new(-4.7, 22., -8.),
        0.045,
        Black,
    );
    h.beam(
        "Generator cable",
        Vec3::new(-4.7, 22., -8.),
        GENERATOR,
        0.045,
        Black,
    );
    h.deco(
        "Signal dial",
        Vec3::new(3.2, 22.9, -7.78),
        Vec3::new(0.75, 0.75, 0.08),
        Cream,
    );
    h.deco(
        "Signal dial needle",
        Vec3::new(3.2, 22.9, -7.71),
        Vec3::new(0.04, 0.5, 0.03),
        Red,
    );
    table(
        h,
        Vec3::new(-10.4, FLOOR_Y + 4. * STOREY, -8.6),
        Vec3::new(1.1, 0.7, 0.7),
    );
    // Front gable, tiny boarded annex and a drain climbing the facade provide layered scale.
    roof(h, Vec3::new(-8., 7.6, 9.3), 8.5, 4.1, 0.62);
    for i in 0..3 {
        h.rotated(
            "Boarded upper window",
            Vec3::new(13.8, 12.3 + i as f32 * 0.35, 2.18),
            Vec3::new(2.3, 0.15, 0.12),
            Quat::from_rotation_z(0.08 * (i as f32 - 1.)),
            DarkWood,
        );
    }
}
