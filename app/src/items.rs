use macroquad::prelude::*;

use crate::world::Aabb;

pub struct Items {
    pub key_taken: bool,
    pub has_key: bool,
    pub carrying: bool,
}

impl Items {
    pub fn new() -> Self {
        Self {
            key_taken: false,
            has_key: false,
            carrying: false,
        }
    }
}

pub struct PropBox {
    pub pos: Vec3,
    pub vel: Vec3,
    pub is_flying: bool,
}

impl PropBox {
    pub fn new(pos: Vec3) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            is_flying: false,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb::from_center(self.pos, vec3(0.6, 0.6, 0.6))
    }

    pub fn update(&mut self, dt: f32, colliders: &[Aabb]) -> bool {
        if !self.is_flying {
            return false;
        }

        let mut impact = false;
        self.vel.y -= 18.0 * dt;

        let substeps = 2;
        let sdt = dt / substeps as f32;

        for _ in 0..substeps {
            self.pos.x += self.vel.x * sdt;
            let mut box_aabb = self.aabb();
            for collider in colliders {
                if box_aabb.overlaps(collider) {
                    if self.vel.x > 0.0 {
                        self.pos.x = collider.min.x - 0.301;
                    } else if self.vel.x < 0.0 {
                        self.pos.x = collider.max.x + 0.301;
                    }
                    self.vel.x = -self.vel.x * 0.35;
                    impact = true;
                    break;
                }
            }

            self.pos.z += self.vel.z * sdt;
            box_aabb = self.aabb();
            for collider in colliders {
                if box_aabb.overlaps(collider) {
                    if self.vel.z > 0.0 {
                        self.pos.z = collider.min.z - 0.301;
                    } else if self.vel.z < 0.0 {
                        self.pos.z = collider.max.z + 0.301;
                    }
                    self.vel.z = -self.vel.z * 0.35;
                    impact = true;
                    break;
                }
            }

            self.pos.y += self.vel.y * sdt;
            box_aabb = self.aabb();
            for collider in colliders {
                if box_aabb.overlaps(collider) {
                    if self.vel.y < 0.0 {
                        self.pos.y = collider.max.y + 0.301;
                        if self.vel.y.abs() > 1.2 {
                            impact = true;
                            self.vel.y = -self.vel.y * 0.25;
                        } else {
                            self.vel.y = 0.0;
                        }
                    } else if self.vel.y > 0.0 {
                        self.pos.y = collider.min.y - 0.301;
                        self.vel.y = 0.0;
                        impact = true;
                    }
                    break;
                }
            }
        }

        if self.pos.y <= 0.3 {
            self.pos.y = 0.3;
            if self.vel.y.abs() > 1.2 {
                impact = true;
            }
            self.vel.y = 0.0;
        }

        self.vel.x *= 0.98;
        self.vel.z *= 0.98;

        if self.vel.length_squared() < 0.1 && (self.pos.y - 0.3).abs() < 0.05 {
            self.vel = Vec3::ZERO;
            self.is_flying = false;
        }

        impact
    }
}
