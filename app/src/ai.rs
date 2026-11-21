//! A finite-state neighbor with occluded sight, hearing and graph-based navigation.
use crate::{
    player::Player,
    world::{HousePlan, NEIGHBOR_SPAWN, line_of_sight},
};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertState {
    #[default]
    Patrol,
    Investigate,
    Chase,
    Search,
}
impl AlertState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Patrol => "PATROLLING",
            Self::Investigate => "HEARD SOMETHING",
            Self::Chase => "HE SPOTTED YOU",
            Self::Search => "SEARCHING",
        }
    }
}
#[derive(Component)]
pub struct Neighbor {
    pub body: Player,
    pub state: AlertState,
    pub awareness: f32,
    pub facing: Vec3,
    pub target: Vec3,
    pub walking: bool,
    pub last_capture: Option<Vec3>,
    route: Vec<Vec3>,
    waypoint: usize,
    patrol_index: usize,
    repath_in: f32,
    lost_for: f32,
    search_for: f32,
}
impl Default for Neighbor {
    fn default() -> Self {
        Self {
            body: Player::new(NEIGHBOR_SPAWN),
            state: AlertState::Patrol,
            awareness: 0.,
            facing: Vec3::Z,
            target: NEIGHBOR_SPAWN,
            walking: false,
            last_capture: None,
            route: vec![],
            waypoint: 0,
            patrol_index: 0,
            repath_in: 0.,
            lost_for: 0.,
            search_for: 0.,
        }
    }
}
impl Neighbor {
    pub fn hear(&mut self, pos: Vec3, loudness: f32, colliders: &[crate::world::Aabb]) {
        if self.state == AlertState::Chase
            || self.body.pos.distance_squared(pos) >= loudness * loudness
        {
            return;
        }
        let wall_factor = if line_of_sight(self.body.eye(), pos + Vec3::Y, colliders) {
            1.
        } else {
            0.45
        };
        if self.state != AlertState::Chase && self.body.pos.distance(pos) < loudness * wall_factor {
            self.state = AlertState::Investigate;
            self.target = pos;
            self.search_for = 7.;
            self.repath_in = 0.;
            self.awareness = self.awareness.max(0.2);
        }
    }
    pub fn tick(
        &mut self,
        dt: f32,
        player: &Player,
        plan: &HousePlan,
        colliders: &[crate::world::Aabb],
    ) -> bool {
        let delta = player.eye() - self.body.eye();
        let distance = delta.length();
        let on_property = player.pos.z < 17.0;
        let in_cone = distance < 2.2 || delta.normalize_or_zero().dot(self.facing) > 0.35;
        let range = if player.crouching { 10.0 } else { 18.0 };
        let visible = on_property
            && distance < range
            && in_cone
            && line_of_sight(self.body.eye(), player.eye(), colliders);
        if visible {
            self.awareness =
                (self.awareness + dt * if player.crouching { 0.5 } else { 1.15 }).min(1.);
            self.target = player.pos;
            self.lost_for = 0.;
            self.search_for = 7.;
            if self.awareness >= 0.95 || distance < 3. {
                if self.state != AlertState::Chase {
                    self.repath_in = 0.;
                }
                self.state = AlertState::Chase;
            } else {
                self.state = AlertState::Investigate;
            }
        } else {
            self.awareness = (self.awareness - dt * 0.13).max(0.);
            self.lost_for += dt;
            if self.state == AlertState::Chase && self.lost_for > 2.4 {
                self.state = AlertState::Search;
                self.search_for = 7.;
            }
            if matches!(self.state, AlertState::Search | AlertState::Investigate) {
                self.search_for -= dt;
                if self.search_for <= 0. {
                    self.state = AlertState::Patrol;
                    self.repath_in = 0.;
                }
            }
        }
        if self.state == AlertState::Patrol
            && let Some(&id) = plan
                .patrol
                .get(self.patrol_index % plan.patrol.len().max(1))
        {
            self.target = if self.patrol_index.is_multiple_of(3) {
                self.last_capture.unwrap_or(plan.navigation[id].pos)
            } else {
                plan.navigation[id].pos
            };
            if flat_distance(self.body.pos, self.target) < 0.7
                && (self.body.pos.y - self.target.y).abs() < 0.7
            {
                self.patrol_index += 1;
                self.repath_in = 0.;
            }
        }
        self.repath_in -= dt;
        let direct = (self.target.y - self.body.pos.y).abs() < 0.45
            && clear_body_route(&self.body, self.target, colliders);
        if !direct && self.repath_in <= 0. {
            self.route = plan.route(self.body.pos, self.target);
            // Replanning midway through a long corridor must not send the
            // neighbor back to the node he just left every 0.9 seconds.
            self.waypoint = usize::from(self.route.get(1).is_some_and(|&next| {
                (next.y - self.body.pos.y).abs() < 0.5
                    && clear_body_route(&self.body, next, colliders)
            }));
            self.repath_in = 0.9;
        }
        let target = if direct {
            self.target
        } else {
            while self.waypoint < self.route.len()
                && flat_distance(self.body.pos, self.route[self.waypoint]) < 0.5
                && (self.body.pos.y - self.route[self.waypoint].y).abs() < 0.5
            {
                self.waypoint += 1;
            }
            self.route
                .get(self.waypoint)
                .copied()
                .unwrap_or(self.body.pos)
        };
        let to = Vec3::new(target.x - self.body.pos.x, 0., target.z - self.body.pos.z);
        self.walking = to.length() > 0.18;
        if self.walking {
            self.facing = to.normalize();
            let speed = match self.state {
                AlertState::Patrol => 1.7,
                AlertState::Investigate | AlertState::Search => 2.2,
                AlertState::Chase => 5.3,
            };
            let movement = self.facing * (speed * dt).min(to.length());
            self.body.vel.y = (self.body.vel.y - 20. * dt).max(-25.);
            self.body
                .translate(movement + Vec3::Y * self.body.vel.y * dt, colliders, 0.26);
            if self.body.on_ground {
                self.body.vel.y = 0.;
            }
        } else {
            self.body.vel.y = (self.body.vel.y - 20. * dt).max(-25.);
            self.body
                .translate(Vec3::Y * self.body.vel.y * dt, colliders, 0.26);
            if self.body.on_ground {
                self.body.vel.y = 0.;
            }
        }
        on_property && visible && distance < 1.35
    }
    pub fn reset(&mut self, last_capture: Option<Vec3>) {
        *self = Self {
            last_capture,
            ..Self::default()
        };
    }
}
fn flat_distance(a: Vec3, b: Vec3) -> f32 {
    Vec2::new(a.x - b.x, a.z - b.z).length()
}
// Seeing a destination around a corner does not mean a full-width body fits.
// Use an expanded obstacle segment for movement, keeping optical LOS for sight.
fn clear_body_route(body: &Player, target: Vec3, colliders: &[crate::world::Aabb]) -> bool {
    !colliders.iter().any(|bounds| {
        if bounds.max.y <= body.pos.y + 0.26 || bounds.min.y >= body.pos.y + body.height() {
            return false;
        }
        let margin = crate::player::PLAYER_HALF_WIDTH + 0.06;
        let expanded = crate::world::Aabb {
            min: Vec3::new(bounds.min.x - margin, -1000., bounds.min.z - margin),
            max: Vec3::new(bounds.max.x + margin, 1000., bounds.max.z + margin),
        };
        crate::world::segment_hit(body.pos, target, &expanded).is_some()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Aabb;
    #[test]
    fn detects_in_front_but_not_through_walls_or_across_safe_street() {
        let plan = HousePlan::default();
        let mut n = Neighbor::default();
        n.body.pos = Vec3::ZERO;
        let p = Player::new(Vec3::new(0., 0., 4.));
        let floor = Aabb::from_center(Vec3::new(0., -0.25, 0.), Vec3::new(40., 0.5, 40.));
        for _ in 0..60 {
            n.tick(1. / 60., &p, &plan, &[floor]);
        }
        assert_eq!(n.state, AlertState::Chase);
        let mut n = Neighbor::default();
        n.body.pos = Vec3::ZERO;
        let walls = [Aabb::from_center(
            Vec3::new(0., 1., 2.),
            Vec3::new(8., 4., 0.5),
        )];
        for _ in 0..10 {
            n.tick(1. / 60., &p, &plan, &walls);
        }
        assert_eq!(n.awareness, 0.);
        let p = Player::new(Vec3::new(0., 0., 24.));
        assert!(!n.tick(1. / 60., &p, &plan, &[]));
    }
    #[test]
    fn noise_changes_patrol_to_investigation() {
        let mut n = Neighbor::default();
        let pos = n.body.pos + Vec3::X * 3.;
        n.hear(pos, 10., &[]);
        assert_eq!(n.state, AlertState::Investigate);
        assert_eq!(n.target, pos);
    }
    #[test]
    fn neighbor_can_follow_the_stairs_and_navigate_to_a_distant_underground_room() {
        let plan = HousePlan::default();
        let mut neighbor = Neighbor::default();
        neighbor.body.pos = Vec3::new(-8., crate::world::FLOOR_Y, -4.4);
        let player = Player::new(crate::world::PLAYER_SPAWN);
        for destination in [
            crate::basement::cell_center(5, 5),
            Vec3::new(-8., crate::world::FLOOR_Y, -4.4),
            crate::basement::cell_center(0, 0),
        ] {
            neighbor.state = AlertState::Investigate;
            neighbor.target = destination;
            neighbor.search_for = 1000.;
            neighbor.repath_in = 0.;
            let mut arrived = false;
            for _ in 0..12000 {
                neighbor.tick(1. / 60., &player, &plan, &plan.colliders);
                if flat_distance(neighbor.body.pos, destination) < 0.6
                    && (neighbor.body.pos.y - destination.y).abs() < 0.6
                {
                    arrived = true;
                    break;
                }
            }
            assert!(
                arrived,
                "neighbor stuck at {:?}, heading to {destination:?}",
                neighbor.body.pos
            );
        }
    }
}
