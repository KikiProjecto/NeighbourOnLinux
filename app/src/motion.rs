//! Render interpolation only; authoritative bodies/collision stay at 60 Hz.
use crate::{ai::Neighbor, player::Player};
use bevy::prelude::*;

#[derive(Component)]
pub(crate) struct MotionHistory {
    previous: Vec3,
}
impl MotionHistory {
    pub(crate) fn new(position: Vec3) -> Self {
        Self { previous: position }
    }
    pub(crate) fn position(&self, current: Vec3, alpha: f32) -> Vec3 {
        // Ladders, retry and fall recovery teleport, rather than gliding through walls.
        if self.previous.distance_squared(current) > 1. {
            current
        } else {
            self.previous.lerp(current, alpha.clamp(0., 1.))
        }
    }
}
pub(crate) fn record_movement(
    mut players: Query<(&Player, &mut MotionHistory), Without<Neighbor>>,
    mut neighbors: Query<(&Neighbor, &mut MotionHistory), Without<Player>>,
) {
    for (player, mut history) in &mut players {
        history.previous = player.pos;
    }
    for (neighbor, mut history) in &mut neighbors {
        history.previous = neighbor.body.pos;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn renders_between_fixed_positions_without_changing_the_body() {
        let history = MotionHistory::new(Vec3::ZERO);
        let body = Vec3::new(0.1, 0., 0.);
        for alpha in [0., 0.25, 0.5, 0.75, 1.] {
            assert!((history.position(body, alpha).x - 0.1 * alpha).abs() < 0.00001);
        }
        assert_eq!(body, Vec3::X * 0.1);
    }
    #[test]
    fn ladder_retry_and_fall_recovery_snap_to_the_new_location() {
        let history = MotionHistory::new(Vec3::ZERO);
        for destination in [Vec3::Y * 3.6, Vec3::Z * 28., Vec3::Y * -15.] {
            assert_eq!(history.position(destination, 0.2), destination);
        }
    }
}
