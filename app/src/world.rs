//! Shared collision, authored geometry, room volumes, and a small navigation graph.
//! Everything is Y-up in meters. Geometry is data; Bevy owns its rendered entities.
use bevy::prelude::*;
use std::collections::VecDeque;

pub const STOREY: f32 = 3.6;
pub const FLOOR_Y: f32 = 0.12;
pub const PLAYER_SPAWN: Vec3 = Vec3::new(-2.0, 0.0, 28.0);
pub const NEIGHBOR_SPAWN: Vec3 = Vec3::new(-4.0, FLOOR_Y, 3.0);
pub const BASEMENT_DOOR: Vec3 = Vec3::new(-8.0, 1.4, -5.0);
pub const FREEZER: Vec3 = Vec3::new(-8.8, 1.0, -8.4);
pub const GENERATOR: Vec3 = Vec3::new(-8.0, 15.4, -7.0);
pub const WINDMILL_SWITCH: Vec3 = Vec3::new(2.0, 22.6, -4.8);
pub const TRAIN_SWITCH: Vec3 = Vec3::new(-8.2, 12.0, 3.0);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}
impl Aabb {
    pub fn from_center(center: Vec3, size: Vec3) -> Self {
        Self {
            min: center - size * 0.5,
            max: center + size * 0.5,
        }
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        self.min.cmplt(other.max).all() && self.max.cmpgt(other.min).all()
    }
    pub fn contains(&self, point: Vec3) -> bool {
        point.cmpge(self.min).all() && point.cmple(self.max).all()
    }
}

/// Normalized segment intersection; parallel rays and rays beginning inside are handled.
pub fn segment_hit(from: Vec3, to: Vec3, bounds: &Aabb) -> Option<f32> {
    let direction = to - from;
    let mut near: f32 = 0.0;
    let mut far: f32 = 1.0;
    for axis in 0..3 {
        if direction[axis].abs() < 1e-6 {
            if from[axis] < bounds.min[axis] || from[axis] > bounds.max[axis] {
                return None;
            }
        } else {
            let a = (bounds.min[axis] - from[axis]) / direction[axis];
            let b = (bounds.max[axis] - from[axis]) / direction[axis];
            near = near.max(a.min(b));
            far = far.min(a.max(b));
            if near > far {
                return None;
            }
        }
    }
    Some(near)
}
pub fn line_of_sight(from: Vec3, to: Vec3, colliders: &[Aabb]) -> bool {
    // Most authored solids are nowhere near a short sight/sound ray. Reject
    // them with six scalar bounds checks before doing the segment/AABB test.
    // Keep touching bounds as candidates: segment_hit treats boundaries as hits.
    let ray_min = from.min(to);
    let ray_max = from.max(to);
    !colliders.iter().any(|bounds| {
        if bounds.max.cmplt(ray_min).any() || bounds.min.cmpgt(ray_max).any() {
            return false;
        }
        segment_hit(from, to, bounds).is_some()
    })
}

#[derive(Clone, Copy, Debug)]
#[repr(usize)]
pub enum Paint {
    Blue,
    PaleBlue,
    Teal,
    Cream,
    White,
    Roof,
    Wood,
    DarkWood,
    Red,
    Yellow,
    Grass,
    Road,
    Concrete,
    Leaf,
    Trunk,
    Metal,
    Glass,
    Black,
    Orange,
    Purple,
    Wallpaper,
    Skin,
    Hair,
    Vest,
    Pants,
    Glow,
}
impl Paint {
    pub const ALL: [Self; 26] = [
        Self::Blue,
        Self::PaleBlue,
        Self::Teal,
        Self::Cream,
        Self::White,
        Self::Roof,
        Self::Wood,
        Self::DarkWood,
        Self::Red,
        Self::Yellow,
        Self::Grass,
        Self::Road,
        Self::Concrete,
        Self::Leaf,
        Self::Trunk,
        Self::Metal,
        Self::Glass,
        Self::Black,
        Self::Orange,
        Self::Purple,
        Self::Wallpaper,
        Self::Skin,
        Self::Hair,
        Self::Vest,
        Self::Pants,
        Self::Glow,
    ];
    pub fn color(self) -> Color {
        let [r, g, b] = match self {
            Self::Blue => [72, 133, 172],
            Self::PaleBlue => [105, 166, 191],
            Self::Teal => [48, 111, 125],
            Self::Cream => [223, 204, 152],
            Self::White => [242, 226, 183],
            Self::Roof => [91, 60, 48],
            Self::Wood => [155, 106, 62],
            Self::DarkWood => [83, 53, 38],
            Self::Red => [167, 48, 38],
            Self::Yellow => [230, 174, 54],
            Self::Grass => [58, 86, 61],
            Self::Road => [35, 43, 50],
            Self::Concrete => [162, 164, 150],
            Self::Leaf => [47, 83, 62],
            Self::Trunk => [93, 72, 46],
            Self::Metal => [111, 127, 125],
            Self::Glass => [90, 167, 183],
            Self::Black => [30, 36, 39],
            Self::Orange => [196, 97, 46],
            Self::Purple => [104, 78, 112],
            Self::Wallpaper => [197, 167, 113],
            Self::Skin => [223, 166, 110],
            Self::Hair => [75, 43, 26],
            Self::Vest => [84, 135, 155],
            Self::Pants => [87, 63, 43],
            Self::Glow => [255, 214, 121],
        };
        Color::srgb_u8(r, g, b)
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Shape {
    Box,
    Sphere,
    Cylinder,
    Cone,
}
#[derive(Clone, Debug)]
pub struct Piece {
    pub name: &'static str,
    pub transform: Transform,
    pub paint: Paint,
    pub shape: Shape,
}
#[derive(Clone, Debug)]
pub struct Room {
    pub name: &'static str,
    pub bounds: Aabb,
}
#[derive(Clone, Debug)]
pub struct NavNode {
    pub pos: Vec3,
    pub links: Vec<usize>,
}

#[derive(Resource)]
pub struct HousePlan {
    pub pieces: Vec<Piece>,
    pub colliders: Vec<Aabb>,
    pub rooms: Vec<Room>,
    pub navigation: Vec<NavNode>,
    pub patrol: Vec<usize>,
    /// First piece belonging to the underground geometry, authored after the house.
    pub underground_start: usize,
}
impl Default for HousePlan {
    fn default() -> Self {
        let mut plan = Self {
            pieces: vec![],
            colliders: vec![],
            rooms: vec![],
            navigation: vec![],
            patrol: vec![],
            underground_start: 0,
        };
        crate::house::build(&mut plan);
        plan
    }
}
impl HousePlan {
    pub fn block(
        &mut self,
        name: &'static str,
        center: Vec3,
        size: Vec3,
        paint: Paint,
        solid: bool,
    ) {
        self.pieces.push(Piece {
            name,
            transform: Transform::from_translation(center).with_scale(size),
            paint,
            shape: Shape::Box,
        });
        if solid {
            self.colliders.push(Aabb::from_center(center, size));
        }
    }
    pub fn deco(&mut self, name: &'static str, center: Vec3, size: Vec3, paint: Paint) {
        self.block(name, center, size, paint, false);
    }
    pub fn rotated(
        &mut self,
        name: &'static str,
        center: Vec3,
        size: Vec3,
        rotation: Quat,
        paint: Paint,
    ) {
        self.pieces.push(Piece {
            name,
            transform: Transform::from_translation(center)
                .with_rotation(rotation)
                .with_scale(size),
            paint,
            shape: Shape::Box,
        });
    }
    pub fn sphere(&mut self, name: &'static str, pos: Vec3, scale: Vec3, paint: Paint) {
        self.pieces.push(Piece {
            name,
            transform: Transform::from_translation(pos).with_scale(scale),
            paint,
            shape: Shape::Sphere,
        });
    }
    pub fn beam(&mut self, name: &'static str, start: Vec3, end: Vec3, width: f32, paint: Paint) {
        self.rotated(
            name,
            (start + end) * 0.5,
            Vec3::new(width, start.distance(end), width),
            Quat::from_rotation_arc(Vec3::Y, (end - start).normalize()),
            paint,
        );
    }
    pub fn pipe(&mut self, start: Vec3, end: Vec3, radius: f32, paint: Paint) {
        self.pieces.push(Piece {
            name: "Exposed plumbing",
            transform: Transform::from_translation((start + end) * 0.5)
                .with_rotation(Quat::from_rotation_arc(Vec3::Y, (end - start).normalize()))
                .with_scale(Vec3::new(radius, start.distance(end), radius)),
            paint,
            shape: Shape::Cylinder,
        });
    }
    pub fn room(&mut self, name: &'static str, center: Vec3, size: Vec3) {
        self.rooms.push(Room {
            name,
            bounds: Aabb::from_center(center, size),
        });
    }
    pub fn room_at(&self, pos: Vec3) -> &'static str {
        self.rooms
            .iter()
            .rev()
            .find(|r| r.bounds.contains(pos))
            .map_or("Raven Brooks · Friendly Court", |r| r.name)
    }
    pub fn node(&mut self, pos: Vec3) -> usize {
        let id = self.navigation.len();
        self.navigation.push(NavNode { pos, links: vec![] });
        id
    }
    pub fn link(&mut self, a: usize, b: usize) {
        self.navigation[a].links.push(b);
        self.navigation[b].links.push(a);
    }
    pub fn nearest_node(&self, pos: Vec3) -> Option<usize> {
        self.navigation
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| nav_distance(a.pos, pos).total_cmp(&nav_distance(b.pos, pos)))
            .map(|(id, _)| id)
    }
    pub fn route(&self, start: Vec3, end: Vec3) -> Vec<Vec3> {
        let (Some(start_id), Some(end_id)) = (self.nearest_node(start), self.nearest_node(end))
        else {
            return vec![];
        };
        let mut previous = vec![None; self.navigation.len()];
        let mut queue = VecDeque::from([start_id]);
        previous[start_id] = Some(start_id);
        while let Some(id) = queue.pop_front() {
            if id == end_id {
                break;
            }
            for &next in &self.navigation[id].links {
                if previous[next].is_none() {
                    previous[next] = Some(id);
                    queue.push_back(next);
                }
            }
        }
        if previous[end_id].is_none() {
            return vec![];
        }
        let mut route = vec![];
        let mut current = end_id;
        while current != start_id {
            route.push(self.navigation[current].pos);
            current = previous[current].expect("visited navigation node has a predecessor");
        }
        route.push(self.navigation[start_id].pos);
        route.reverse();
        route
    }
}
fn nav_distance(a: Vec3, b: Vec3) -> f32 {
    let d = a - b;
    d.x * d.x + d.z * d.z + d.y * d.y * 6.0
}

#[derive(Resource, Default)]
pub struct CollisionWorld {
    pub solids: Vec<Aabb>,
    pub static_count: usize,
    pub loose_boxes: Vec<(Entity, Aabb)>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn broad_phase_line_of_sight_matches_full_segment_checks() {
        let mut boxes = Vec::new();
        let mut seed = 0x51a7_10c3_u32;
        for _ in 0..1000 {
            let mut next = || {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed as f32 / u32::MAX as f32) * 160. - 80.
            };
            let center = Vec3::new(next(), next(), next());
            let size = Vec3::new(
                next().abs() * 0.12,
                next().abs() * 0.12,
                next().abs() * 0.12,
            );
            boxes.push(Aabb::from_center(center, size));
        }
        // Include wide floor/wall-like solids and edge-touching cases.
        boxes.push(Aabb::from_center(Vec3::ZERO, Vec3::new(320., 0.6, 320.)));
        for (from, to) in [
            (Vec3::ZERO, Vec3::ZERO),
            (Vec3::new(-2., 0., 0.), Vec3::new(2., 0., 0.)),
            (Vec3::new(0., 2., -3.), Vec3::new(0., 2., 3.)),
            (Vec3::new(100., 100., 100.), Vec3::new(110., 105., 108.)),
        ] {
            let expected = !boxes
                .iter()
                .any(|bounds| segment_hit(from, to, bounds).is_some());
            assert_eq!(line_of_sight(from, to, &boxes), expected);
        }
        for _ in 0..500 {
            let mut next = || {
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                (seed as f32 / u32::MAX as f32) * 160. - 80.
            };
            let from = Vec3::new(next(), next(), next());
            let to = Vec3::new(next(), next(), next());
            let expected = !boxes
                .iter()
                .any(|bounds| segment_hit(from, to, bounds).is_some());
            assert_eq!(line_of_sight(from, to, &boxes), expected);
        }
    }
    #[test]
    fn ray_handles_parallel_inside_and_missed_segments() {
        let a = Aabb::from_center(Vec3::ZERO, Vec3::splat(2.0));
        assert!(segment_hit(Vec3::new(-3., 0., 0.), Vec3::new(3., 0., 0.), &a).is_some());
        assert!(segment_hit(Vec3::ZERO, Vec3::Y, &a).is_some());
        assert!(segment_hit(Vec3::new(-3., 3., 0.), Vec3::new(3., 3., 0.), &a).is_none());
        assert!(segment_hit(Vec3::new(0., 0., 3.), Vec3::new(0., 1., 3.), &a).is_none());
    }
    #[test]
    fn navigation_connects_yard_to_every_floor() {
        let plan = HousePlan::default();
        for level in 0..=5 {
            let route = plan.route(
                PLAYER_SPAWN,
                Vec3::new(3., FLOOR_Y + level as f32 * STOREY, 5.),
            );
            assert!(!route.is_empty(), "floor {level} is disconnected");
        }
        assert!(plan.rooms.len() >= 15);
        assert!(plan.pieces.len() > 1200);
    }
}
