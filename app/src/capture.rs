use macroquad::prelude::*;
use macroquad::window::get_internal_gl;

use crate::ai::{AlertState, Neighbor};
use crate::game::{GameState, Status};
use crate::player::Player;
use crate::world::Aabb;

pub fn save_screenshot(path: &str, game: &GameState) {
    let target = render_target_ex(
        1280,
        720,
        RenderTargetParams {
            depth: true,
            ..Default::default()
        },
    );
    game.draw_scene(Some(target.clone()));
    let hud = Camera2D::from_display_rect(Rect::new(0.0, 0.0, 1280.0, 720.0));
    set_camera(&Camera2D {
        render_target: Some(target.clone()),
        ..hud
    });
    crate::ui::draw(game, 1280.0, 720.0);
    unsafe { get_internal_gl().flush() };
    target.texture.get_texture_data().export_png(path);
}

pub fn self_test() {
    test_wall_collision();
    test_stair_step();
    test_ai_states();
    test_item_pickup();
    test_carry_drop();
    println!("self-test ok");
}

fn test_wall_collision() {
    let ground = Aabb::from_center(vec3(0.0, -0.25, 0.0), vec3(20.0, 0.5, 20.0));
    let wall = Aabb::from_center(vec3(0.0, 1.5, 0.0), vec3(1.0, 3.0, 4.0));
    let colliders = [ground, wall];
    let mut player = Player::new(vec3(-3.0, 0.0, 0.0));

    player.translate(vec3(10.0, 0.0, 0.0), &colliders, 0.45);

    assert!(
        player.pos.x < -0.8 && player.pos.x > -1.0,
        "player crossed wall: x={}",
        player.pos.x
    );
}

fn test_stair_step() {
    let ground = Aabb::from_center(vec3(0.0, -0.25, 0.0), vec3(20.0, 0.5, 20.0));
    let step = Aabb::from_center(vec3(0.0, 0.15, 0.0), vec3(2.0, 0.3, 4.0));
    let colliders = [ground, step];
    let mut player = Player::new(vec3(-3.0, 0.0, 0.0));

    player.translate(vec3(6.0, 0.0, 0.0), &colliders, 0.45);

    assert!(
        player.pos.y >= 0.29,
        "player did not step up: y={}",
        player.pos.y
    );
}

fn test_ai_states() {
    let colliders: [Aabb; 0] = [];
    let mut neighbor = Neighbor::new(vec3(0.0, 0.0, 0.0), vec![vec3(0.0, 0.0, 0.0)]);
    let far = vec3(0.0, 0.0, 100.0);
    let visible = vec3(0.0, 0.0, 8.0);

    assert_eq!(neighbor.state, AlertState::Patrol);
    neighbor.update(0.1, far, &colliders);
    assert_eq!(neighbor.state, AlertState::Patrol);

    neighbor.update(0.1, visible, &colliders);
    assert_eq!(neighbor.state, AlertState::Suspicious);

    neighbor.update(1.0, visible, &colliders);
    assert_eq!(neighbor.state, AlertState::Chase);

    neighbor.update(3.0, far, &colliders);
    assert_eq!(neighbor.state, AlertState::Suspicious);

    neighbor.update(3.0, far, &colliders);
    assert_eq!(neighbor.state, AlertState::Patrol);
}

fn test_item_pickup() {
    let mut game = GameState::new();
    game.player.pos = vec3(game.world.key_pos.x, 0.0, game.world.key_pos.z);
    game.interact();
    assert!(game.items.has_key, "key not picked up");
    assert_eq!(game.status, Status::Playing);

    game.player.pos = vec3(game.world.door_center.x, 0.0, game.world.door_center.z - 1.5);
    game.interact();
    assert_eq!(game.status, Status::Won);
    assert!(game.world.door_open);
}

fn test_carry_drop() {
    let mut game = GameState::new();
    game.player.pos = vec3(game.world.object_pos.x, 0.0, game.world.object_pos.z);
    game.interact();
    assert!(game.items.carrying, "object not picked up");
    game.interact();
    assert!(!game.items.carrying, "object not dropped");
}
