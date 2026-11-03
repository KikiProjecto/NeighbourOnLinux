use macroquad::prelude::*;

use crate::world::{line_of_sight, Aabb};

const DETECT_RANGE: f32 = 14.0;
const PATROL_SPEED: f32 = 1.9;
const CHASE_SPEED: f32 = 3.4;
const SUSPICION_TO_CHASE: f32 = 0.8;
const LOST_SIGHT_TO_SUSPICIOUS: f32 = 2.5;
const LOST_SIGHT_TO_PATROL: f32 = 3.0;
const CATCH_RADIUS: f32 = 1.2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertState {
    Patrol,
    Suspicious,
    Chase,
}

pub struct Neighbor {
    pub pos: Vec3,
    pub state: AlertState,
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
            waypoints,
            waypoint_index: 0,
            suspicion_timer: 0.0,
            lost_timer: 0.0,
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
                    self.patrol(dt);
                }
            }
            AlertState::Suspicious => {
                if visible {
                    self.suspicion_timer += dt;
                    self.lost_timer = 0.0;
                    if self.suspicion_timer >= SUSPICION_TO_CHASE || distance < 3.0 {
                        self.state = AlertState::Chase;
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
                } else {
                    self.lost_timer += dt;
                    if self.lost_timer >= LOST_SIGHT_TO_SUSPICIOUS {
                        self.state = AlertState::Suspicious;
                        self.suspicion_timer = 0.0;
                        self.lost_timer = 0.0;
                    }
                }
                self.move_towards(player_pos, dt, CHASE_SPEED);
            }
        }

        visible && distance < CATCH_RADIUS
    }

    fn patrol(&mut self, dt: f32) {
        let target = self.waypoints[self.waypoint_index];
        if (target - self.pos).length() < 0.5 {
            self.waypoint_index = (self.waypoint_index + 1) % self.waypoints.len();
            return;
        }
        self.move_towards(target, dt, PATROL_SPEED);
    }

    fn move_towards(&mut self, target: Vec3, dt: f32, speed: f32) {
        let flat = vec3(target.x - self.pos.x, 0.0, target.z - self.pos.z);
        if flat.length_squared() > 1e-4 {
            self.pos += flat.normalize() * speed * dt;
        }
    }
}
