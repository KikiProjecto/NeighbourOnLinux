use macroquad::prelude::*;

pub const WALL_THICKNESS: f32 = 0.4;
const STEP_HEIGHT: f32 = 0.3;
const STEP_DEPTH: f32 = 0.5;
const STAIR_COUNT: usize = 10;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    pub fn from_center(center: Vec3, size: Vec3) -> Self {
        let half = size * 0.5;
        Self {
            min: center - half,
            max: center + half,
        }
    }

    pub fn overlaps(&self, other: &Aabb) -> bool {
        self.min.x < other.max.x
            && self.max.x > other.min.x
            && self.min.y < other.max.y
            && self.max.y > other.min.y
            && self.min.z < other.max.z
            && self.max.z > other.min.z
    }

    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
}

pub struct Solid {
    pub aabb: Aabb,
    pub color: Color,
}

pub struct World {
    pub solids: Vec<Solid>,
    pub colliders: Vec<Aabb>,
    pub door_center: Vec3,
    pub door_size: Vec3,
    pub door_open: bool,
    door_collider: Aabb,
    pub key_pos: Vec3,
    pub object_pos: Vec3,
    pub player_start: Vec3,
    pub neighbor_start: Vec3,
    pub waypoints: Vec<Vec3>,
}

impl World {
    pub fn new() -> Self {
        let door_center = vec3(0.0, 1.2, 8.0);
        let door_size = vec3(2.2, 2.4, 0.18);
        let mut world = Self {
            solids: Vec::new(),
            colliders: Vec::new(),
            door_center,
            door_size,
            door_open: false,
            door_collider: Aabb::from_center(door_center, door_size),
            key_pos: vec3(-6.0, 0.86, -5.0),
            object_pos: vec3(5.0, 0.35, 4.0),
            player_start: vec3(0.0, 0.0, 11.0),
            neighbor_start: vec3(0.0, 0.0, 3.0),
            waypoints: vec![
                vec3(-6.0, 0.0, 5.0),
                vec3(6.0, 0.0, 5.0),
                vec3(5.0, 0.0, 1.0),
                vec3(-5.0, 0.0, 1.0),
            ],
        };
        world.build_house();
        world.colliders.push(world.door_collider);
        world
    }

    pub fn open_door(&mut self) {
        self.door_open = true;
        self.colliders.retain(|c| *c != self.door_collider);
    }

    fn add_solid(&mut self, center: Vec3, size: Vec3, color: Color) {
        let aabb = Aabb::from_center(center, size);
        self.solids.push(Solid { aabb, color });
        self.colliders.push(aabb);
    }

    fn add_deco(&mut self, center: Vec3, size: Vec3, color: Color) {
        self.solids.push(Solid {
            aabb: Aabb::from_center(center, size),
            color,
        });
    }

    fn build_house(&mut self) {
        let wall = Color::new(0.74, 0.68, 0.58, 1.0);
        let floor = Color::new(0.42, 0.31, 0.22, 1.0);
        let stairs = Color::new(0.55, 0.41, 0.28, 1.0);
        let roof = Color::new(0.32, 0.34, 0.36, 1.0);
        let ground = Color::new(0.33, 0.45, 0.28, 1.0);
        let furniture = Color::new(0.31, 0.36, 0.40, 1.0);
        let wood = Color::new(0.47, 0.33, 0.22, 1.0);

        self.add_solid(vec3(0.0, -0.25, 0.0), vec3(80.0, 0.5, 80.0), ground);
        self.add_deco(vec3(0.0, 0.02, 0.0), vec3(20.4, 0.04, 16.4), floor);
        self.add_deco(vec3(0.0, 0.06, 9.0), vec3(4.0, 0.12, 2.0), wood);

        // Ground floor shell, with the exit doorway in the front wall.
        self.add_solid(vec3(0.0, 1.5, -8.0), vec3(20.0 + WALL_THICKNESS, 3.0, WALL_THICKNESS), wall);
        self.add_solid(vec3(-10.0, 1.5, 0.0), vec3(WALL_THICKNESS, 3.0, 16.0 + WALL_THICKNESS), wall);
        self.add_solid(vec3(10.0, 1.5, 0.0), vec3(WALL_THICKNESS, 3.0, 16.0 + WALL_THICKNESS), wall);
        self.add_solid(vec3(-5.65, 1.5, 8.0), vec3(9.1, 3.0, WALL_THICKNESS), wall);
        self.add_solid(vec3(5.65, 1.5, 8.0), vec3(9.1, 3.0, WALL_THICKNESS), wall);
        self.add_solid(vec3(0.0, 2.7, 8.0), vec3(2.2, 0.6, WALL_THICKNESS), wall);

        // Partition between living room and kitchen, with an open walkway.
        self.add_solid(vec3(-6.0, 1.5, -2.0), vec3(8.0, 3.0, 0.3), wall);
        self.add_solid(vec3(5.0, 1.5, -2.0), vec3(10.0, 3.0, 0.3), wall);

        // Stairs up along +z, inside the right rear room.
        for i in 0..STAIR_COUNT {
            let top = STEP_HEIGHT * (i + 1) as f32;
            let z = -7.5 + STEP_DEPTH * (i as f32 + 0.5);
            self.add_solid(vec3(7.5, top * 0.5, z), vec3(3.0, top, STEP_DEPTH), stairs);
        }

        // Second floor slab around the stair opening.
        let slab_y = 3.125;
        let slab_t = 0.25;
        self.add_solid(vec3(-2.35, slab_y, 0.0), vec3(15.7, slab_t, 16.4), floor);
        self.add_solid(vec3(9.85, slab_y, 0.0), vec3(0.7, slab_t, 16.4), floor);
        self.add_solid(vec3(7.5, slab_y, 2.75), vec3(4.0, slab_t, 10.5), floor);
        self.add_solid(vec3(7.5, slab_y, -7.85), vec3(4.0, slab_t, 0.7), floor);

        // Upper floor shell and roof.
        self.add_solid(vec3(0.0, 4.625, -8.0), vec3(20.0 + WALL_THICKNESS, 2.75, WALL_THICKNESS), wall);
        self.add_solid(vec3(-10.0, 4.625, 0.0), vec3(WALL_THICKNESS, 2.75, 16.0 + WALL_THICKNESS), wall);
        self.add_solid(vec3(10.0, 4.625, 0.0), vec3(WALL_THICKNESS, 2.75, 16.0 + WALL_THICKNESS), wall);
        self.add_solid(vec3(0.0, 4.625, 8.0), vec3(20.0 + WALL_THICKNESS, 2.75, WALL_THICKNESS), wall);
        self.add_solid(vec3(0.0, 6.15, 0.0), vec3(21.0, 0.3, 17.0), roof);

        self.add_solid(vec3(-6.0, 0.4, -5.0), vec3(2.4, 0.8, 1.2), wood);
        self.add_solid(vec3(5.0, 0.45, 5.5), vec3(3.0, 0.9, 1.1), furniture);
        self.add_solid(vec3(-6.0, 3.65, 4.0), vec3(0.9, 0.8, 0.9), wood);
        self.add_solid(vec3(-6.0, 4.05, 4.0), vec3(0.6, 0.6, 0.6), wood);
    }
}

pub fn line_of_sight(from: Vec3, to: Vec3, colliders: &[Aabb]) -> bool {
    colliders
        .iter()
        .all(|collider| !segment_hits_aabb(from, to, collider))
}

fn segment_hits_aabb(from: Vec3, to: Vec3, aabb: &Aabb) -> bool {
    let direction = to - from;
    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = 1.0;
    let axes = [
        (from.x, direction.x, aabb.min.x, aabb.max.x),
        (from.y, direction.y, aabb.min.y, aabb.max.y),
        (from.z, direction.z, aabb.min.z, aabb.max.z),
    ];
    for (origin, delta, min, max) in axes {
        if delta.abs() < 1e-6 {
            if origin < min || origin > max {
                return false;
            }
        } else {
            let t1 = (min - origin) / delta;
            let t2 = (max - origin) / delta;
            t_min = t_min.max(t1.min(t2));
            t_max = t_max.min(t1.max(t2));
            if t_min > t_max {
                return false;
            }
        }
    }
    true
}
