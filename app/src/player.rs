use bevy::prelude::*;

use crate::world::Aabb;

pub const PLAYER_HALF_WIDTH: f32 = 0.35;
pub const STAND_HEIGHT: f32 = 1.8;
pub const CROUCH_HEIGHT: f32 = 1.1;
const EYE_BELOW_TOP: f32 = 0.15;
const GRAVITY: f32 = 20.0;
const JUMP_SPEED: f32 = 7.2;
const WALK_SPEED: f32 = 4.5;
const RUN_SPEED: f32 = 7.0;
const CROUCH_SPEED: f32 = 2.0;
const GROUND_ACCEL: f32 = 40.0;
const AIR_ACCEL: f32 = 10.0;
const STEP_HEIGHT: f32 = 0.45;
const LOOK_SENSITIVITY: f32 = 0.0025;

pub struct MoveInput {
    pub forward: f32,
    pub strafe: f32,
    pub jump: bool,
    pub run: bool,
    pub crouch: bool,
    pub look: Vec2,
}

#[derive(Component, Debug)]
pub struct Player {
    pub pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,
    pub crouching: bool,
}

impl Player {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            crouching: false,
        }
    }

    pub fn height(&self) -> f32 {
        if self.crouching {
            CROUCH_HEIGHT
        } else {
            STAND_HEIGHT
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb {
            min: self.pos + Vec3::new(-PLAYER_HALF_WIDTH, 0.0, -PLAYER_HALF_WIDTH),
            max: self.pos + Vec3::new(PLAYER_HALF_WIDTH, self.height(), PLAYER_HALF_WIDTH),
        }
    }

    pub fn eye(&self) -> Vec3 {
        self.pos + Vec3::new(0.0, self.height() - EYE_BELOW_TOP, 0.0)
    }

    pub fn forward(&self) -> Vec3 {
        let pitch_cos = self.pitch.cos();
        Vec3::new(
            -self.yaw.sin() * pitch_cos,
            self.pitch.sin(),
            -self.yaw.cos() * pitch_cos,
        )
    }

    pub fn forward_flat(&self) -> Vec3 {
        Vec3::new(-self.yaw.sin(), 0.0, -self.yaw.cos())
    }

    fn right_flat(&self) -> Vec3 {
        Vec3::new(self.yaw.cos(), 0.0, -self.yaw.sin())
    }

    pub fn step(&mut self, dt: f32, input: &MoveInput, colliders: &[Aabb]) {
        self.yaw -= input.look.x * LOOK_SENSITIVITY;
        self.pitch = (self.pitch - input.look.y * LOOK_SENSITIVITY).clamp(-1.45, 1.45);
        // A low ceiling must not allow standing up through solid geometry.
        let was_crouching = self.crouching;
        self.crouching = input.crouch;
        if was_crouching && !input.crouch && colliders.iter().any(|c| self.aabb().overlaps(c)) {
            self.crouching = true;
        }

        let mut wish = self.forward_flat() * input.forward + self.right_flat() * input.strafe;
        if wish.length_squared() > 0.0 {
            wish = wish.normalize();
        }
        let speed = if self.crouching {
            CROUCH_SPEED
        } else if input.run {
            RUN_SPEED
        } else {
            WALK_SPEED
        };
        let accel = if self.on_ground {
            GROUND_ACCEL
        } else {
            AIR_ACCEL
        };
        self.vel.x = approach(self.vel.x, wish.x * speed, accel * dt);
        self.vel.z = approach(self.vel.z, wish.z * speed, accel * dt);
        self.vel.y -= GRAVITY * dt;
        if input.jump && self.on_ground {
            self.vel.y = JUMP_SPEED;
            self.on_ground = false;
        }

        self.on_ground = false;
        self.move_horizontal(Vec3::new(self.vel.x * dt, 0.0, self.vel.z * dt), colliders);
        self.move_vertical(self.vel.y * dt, colliders);
    }

    pub fn translate(&mut self, delta: Vec3, colliders: &[Aabb], step_height: f32) {
        self.move_horizontal_with_step(Vec3::new(delta.x, 0.0, delta.z), colliders, step_height);
        self.on_ground = false;
        self.move_vertical(delta.y, colliders);
    }

    fn move_horizontal(&mut self, delta: Vec3, colliders: &[Aabb]) {
        self.move_horizontal_with_step(delta, colliders, STEP_HEIGHT);
    }

    fn move_horizontal_with_step(&mut self, delta: Vec3, colliders: &[Aabb], step_height: f32) {
        let steps = collision_substeps(delta);
        let delta = delta / steps as f32;
        for _ in 0..steps {
            self.pos.x += delta.x;
            self.resolve_horizontal(0, delta.x, colliders, step_height);
            self.pos.z += delta.z;
            self.resolve_horizontal(2, delta.z, colliders, step_height);
        }
    }

    fn resolve_horizontal(
        &mut self,
        axis: usize,
        amount: f32,
        colliders: &[Aabb],
        step_height: f32,
    ) {
        if amount == 0.0 {
            return;
        }
        for collider in colliders {
            if !self.aabb().overlaps(collider) {
                continue;
            }
            let rise = collider.max.y - self.pos.y;
            if rise > 0.0 && rise <= step_height && self.pos.y + self.height() > collider.max.y {
                let previous_y = self.pos.y;
                // A threshold can consist of stacked soil and floor slabs. Step to
                // the highest overlapping surface within reach in one operation.
                // Testing the lower slab first would reject the still-overlapping
                // upper slab forever, even though the combined rise is small.
                let body = self.aabb();
                self.pos.y = colliders
                    .iter()
                    .filter(|other| {
                        body.overlaps(other)
                            && other.max.y > previous_y
                            && other.max.y - previous_y <= step_height
                    })
                    .map(|other| other.max.y)
                    .fold(collider.max.y, f32::max);
                if !colliders.iter().any(|other| self.aabb().overlaps(other)) {
                    continue;
                }
                self.pos.y = previous_y;
            }
            if axis == 0 {
                if amount > 0.0 {
                    self.pos.x = collider.min.x - PLAYER_HALF_WIDTH - 0.001;
                } else {
                    self.pos.x = collider.max.x + PLAYER_HALF_WIDTH + 0.001;
                }
            } else if amount > 0.0 {
                self.pos.z = collider.min.z - PLAYER_HALF_WIDTH - 0.001;
            } else {
                self.pos.z = collider.max.z + PLAYER_HALF_WIDTH + 0.001;
            }
        }
    }

    fn move_vertical(&mut self, dy: f32, colliders: &[Aabb]) {
        let steps = collision_substeps(Vec3::new(0.0, dy, 0.0));
        let dy = dy / steps as f32;
        for _ in 0..steps {
            self.pos.y += dy;
            for collider in colliders {
                if !self.aabb().overlaps(collider) {
                    continue;
                }
                if dy > 0.0 {
                    self.pos.y = collider.min.y - self.height() - 0.001;
                } else {
                    self.pos.y = collider.max.y;
                    self.on_ground = true;
                }
                self.vel.y = 0.0;
            }
        }
    }
}

fn collision_substeps(delta: Vec3) -> usize {
    ((delta.length() / 0.2).ceil() as usize).max(1)
}

fn approach(current: f32, target: f32, max_delta: f32) -> f32 {
    if current < target {
        (current + max_delta).min(target)
    } else {
        (current - max_delta).max(target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn idle(crouch: bool) -> MoveInput {
        MoveInput {
            forward: 0.,
            strafe: 0.,
            jump: false,
            run: false,
            crouch,
            look: Vec2::ZERO,
        }
    }
    #[test]
    fn crouch_cannot_stand_through_a_low_ceiling() {
        let floor = Aabb::from_center(Vec3::new(0., -0.25, 0.), Vec3::new(10., 0.5, 10.));
        let ceiling = Aabb::from_center(Vec3::new(0., 1.6, 0.), Vec3::new(4., 0.3, 4.));
        let mut p = Player::new(Vec3::ZERO);
        p.crouching = true;
        p.step(1. / 60., &idle(false), &[floor, ceiling]);
        assert!(p.crouching);
        assert!(p.pos.y.abs() < 0.01);
    }
    #[test]
    fn high_speed_translation_does_not_tunnel_through_thin_walls() {
        let wall = Aabb::from_center(Vec3::new(2., 1., 0.), Vec3::new(0.1, 3., 8.));
        let mut p = Player::new(Vec3::ZERO);
        p.translate(Vec3::X * 20., &[wall], 0.);
        assert!(p.pos.x < 1.61);
    }
}
