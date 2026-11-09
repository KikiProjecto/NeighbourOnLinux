use macroquad::prelude::*;

use crate::world::{line_of_sight, Aabb};

const DETECT_RANGE: f32 = 14.0;
const PATROL_SPEED: f32 = 1.9;
const CHASE_SPEED: f32 = 3.4;
const INVESTIGATE_SPEED: f32 = 2.4;
const SUSPICION_TO_CHASE: f32 = 0.8;
const LOST_SIGHT_TO_SUSPICIOUS: f32 = 2.5;
const LOST_SIGHT_TO_PATROL: f32 = 3.5;
const CATCH_RADIUS: f32 = 1.3;
const NEIGHBOR_HALF_WIDTH: f32 = 0.38;
const NEIGHBOR_HEIGHT: f32 = 1.9;
const STEP_HEIGHT: f32 = 0.45;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertState {
    Patrol,
    Suspicious,
    Chase,
}

pub struct Neighbor {
    pub pos: Vec3,
    pub state: AlertState,
    pub last_seen_pos: Option<Vec3>,
    pub investigate_pos: Option<Vec3>,
    waypoints: Vec<Vec3>,
    waypoint_index: usize,
    suspicion_timer: f32,
    lost_timer: f32,
}

impl Neighbor {
    pub fn new(pos: Vec3, waypoints: Vec<Vec3>) -> Self {
        Self {
            pos,
            state: AlertState::Patrol,
            last_seen_pos: None,
            investigate_pos: None,
            waypoints,
            waypoint_index: 0,
            suspicion_timer: 0.0,
            lost_timer: 0.0,
        }
    }

    pub fn aabb(&self) -> Aabb {
        Aabb {
            min: self.pos + vec3(-NEIGHBOR_HALF_WIDTH, 0.0, -NEIGHBOR_HALF_WIDTH),
            max: self.pos + vec3(NEIGHBOR_HALF_WIDTH, NEIGHBOR_HEIGHT, NEIGHBOR_HALF_WIDTH),
        }
    }

    pub fn hear_noise(&mut self, noise_pos: Vec3) {
        if self.state == AlertState::Patrol || self.state == AlertState::Suspicious {
            self.investigate_pos = Some(noise_pos);
            self.state = AlertState::Suspicious;
            self.suspicion_timer = 0.5;
            self.lost_timer = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32, player_pos: Vec3, colliders: &[Aabb]) -> bool {
        let distance = (player_pos - self.pos).length();
        let visible = distance <= DETECT_RANGE
            && line_of_sight(
                self.pos + vec3(0.0, 1.5, 0.0),
                player_pos + vec3(0.0, 1.4, 0.0),
                colliders,
            );

        if visible {
            self.last_seen_pos = Some(player_pos);
        }

        match self.state {
            AlertState::Patrol => {
                if visible {
                    if distance < 5.0 {
                        self.state = AlertState::Chase;
                    } else {
                        self.state = AlertState::Suspicious;
                        self.suspicion_timer = 0.0;
                        self.lost_timer = 0.0;
                    }
                } else {
                    self.patrol(dt, colliders);
                }
            }
            AlertState::Suspicious => {
                if visible {
                    self.suspicion_timer += dt;
                    self.lost_timer = 0.0;
                    if self.suspicion_timer >= SUSPICION_TO_CHASE || distance < 3.2 {
                        self.state = AlertState::Chase;
                    } else {
                        self.move_towards(player_pos, dt, INVESTIGATE_SPEED, colliders);
                    }
                } else if let Some(target) = self.investigate_pos {
                    let d = (vec3(target.x, 0.0, target.z) - vec3(self.pos.x, 0.0, self.pos.z)).length();
                    if d < 0.8 {
                        self.investigate_pos = None;
                    } else {
                        self.move_towards(target, dt, INVESTIGATE_SPEED, colliders);
                    }
                    self.lost_timer += dt;
                    if self.lost_timer >= LOST_SIGHT_TO_PATROL {
                        self.state = AlertState::Patrol;
                        self.investigate_pos = None;
                        self.lost_timer = 0.0;
                    }
                } else {
                    self.lost_timer += dt;
                    if self.lost_timer >= LOST_SIGHT_TO_PATROL {
                        self.state = AlertState::Patrol;
                        self.lost_timer = 0.0;
                    }
                }
            }
            AlertState::Chase => {
                if visible {
                    self.lost_timer = 0.0;
                    self.move_towards(player_pos, dt, CHASE_SPEED, colliders);
                } else {
                    self.lost_timer += dt;
                    if let Some(last_pos) = self.last_seen_pos {
                        let d = (vec3(last_pos.x, 0.0, last_pos.z) - vec3(self.pos.x, 0.0, self.pos.z)).length();
                        if d < 0.8 || self.lost_timer >= LOST_SIGHT_TO_SUSPICIOUS {
                            self.state = AlertState::Suspicious;
                            self.investigate_pos = Some(last_pos);
                            self.last_seen_pos = None;
                            self.suspicion_timer = 0.0;
                        } else {
                            self.move_towards(last_pos, dt, CHASE_SPEED, colliders);
                        }
                    } else if self.lost_timer >= LOST_SIGHT_TO_SUSPICIOUS {
                        self.state = AlertState::Suspicious;
                        self.suspicion_timer = 0.0;
                    }
                }
            }
        }

        (visible && distance < CATCH_RADIUS) || distance < 0.85
    }

    fn patrol(&mut self, dt: f32, colliders: &[Aabb]) {
        if self.waypoints.is_empty() {
            return;
        }
        let target = self.waypoints[self.waypoint_index];
        if (vec3(target.x, 0.0, target.z) - vec3(self.pos.x, 0.0, self.pos.z)).length() < 0.6 {
            self.waypoint_index = (self.waypoint_index + 1) % self.waypoints.len();
            return;
        }
        self.move_towards(target, dt, PATROL_SPEED, colliders);
    }

    pub fn move_towards(&mut self, target: Vec3, dt: f32, speed: f32, colliders: &[Aabb]) {
        let flat = vec3(target.x - self.pos.x, 0.0, target.z - self.pos.z);
        if flat.length_squared() < 1e-4 {
            return;
        }
        let delta = flat.normalize() * speed * dt;
        let substeps = ((delta.length() / 0.15).ceil() as usize).max(1);
        let s_delta = delta / substeps as f32;

        for _ in 0..substeps {
            self.pos.x += s_delta.x;
            self.resolve_axis(0, s_delta.x, colliders);
            self.pos.z += s_delta.z;
            self.resolve_axis(2, s_delta.z, colliders);
        }
    }

    fn resolve_axis(&mut self, axis: usize, amount: f32, colliders: &[Aabb]) {
        if amount == 0.0 {
            return;
        }
        for collider in colliders {
            if !self.aabb().overlaps(collider) {
                continue;
            }
            let rise = collider.max.y - self.pos.y;
            if rise > 0.0 && rise <= STEP_HEIGHT && self.pos.y + NEIGHBOR_HEIGHT > collider.max.y {
                self.pos.y = collider.max.y;
                continue;
            }
            if axis == 0 {
                if amount > 0.0 {
                    self.pos.x = collider.min.x - NEIGHBOR_HALF_WIDTH - 0.001;
                } else {
                    self.pos.x = collider.max.x + NEIGHBOR_HALF_WIDTH + 0.001;
                }
            } else if amount > 0.0 {
                self.pos.z = collider.min.z - NEIGHBOR_HALF_WIDTH - 0.001;
            } else {
                self.pos.z = collider.max.z + NEIGHBOR_HALF_WIDTH + 0.001;
            }
        }
    }
}
