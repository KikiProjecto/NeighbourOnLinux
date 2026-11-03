use macroquad::prelude::*;

use crate::ai::{AlertState, Neighbor};
use crate::items::Items;
use crate::player::{MoveInput, Player};
use crate::ui;
use crate::world::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Playing,
    Won,
    Lost,
}

pub struct GameState {
    pub player: Player,
    pub world: World,
    pub neighbor: Neighbor,
    pub items: Items,
    pub status: Status,
    pub message: String,
    pub message_timer: f32,
}

impl GameState {
    pub fn new() -> Self {
        let world = World::new();
        let player = Player::new(world.player_start);
        let neighbor = Neighbor::new(world.neighbor_start, world.waypoints.clone());
        Self {
            player,
            world,
            neighbor,
            items: Items::new(),
            status: Status::Playing,
            message: String::new(),
            message_timer: 0.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        if self.message_timer > 0.0 {
            self.message_timer -= dt;
        }
        if self.status != Status::Playing {
            if is_key_pressed(KeyCode::R) {
                *self = GameState::new();
            }
            return;
        }

        let input = MoveInput {
            forward: axis(KeyCode::W, KeyCode::S),
            strafe: axis(KeyCode::D, KeyCode::A),
            jump: is_key_pressed(KeyCode::Space),
            run: is_key_down(KeyCode::LeftShift) || is_key_down(KeyCode::RightShift),
            crouch: is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl),
            look: mouse_delta_position(),
        };
        self.player.step(dt, &input, &self.world.colliders);

        if is_key_pressed(KeyCode::E) {
            self.interact();
        }
        if self.status != Status::Playing {
            return;
        }
        if self.neighbor.update(dt, self.player.pos, &self.world.colliders) {
            self.status = Status::Lost;
        }
    }

    pub fn interact(&mut self) {
        if self.items.carrying {
            self.items.carrying = false;
            let forward = self.player.forward_flat();
            self.world.object_pos =
                vec3(self.player.pos.x + forward.x, 0.35, self.player.pos.z + forward.z);
            self.set_message("Kotak dilepas.");
            return;
        }

        if !self.items.key_taken
            && self.player.pos.distance(self.world.key_pos) < 2.0
        {
            self.items.key_taken = true;
            self.items.has_key = true;
            self.set_message("Kunci diambil.");
            return;
        }

        if self.distance_to_door() < 2.4 {
            if self.items.has_key {
                self.world.open_door();
                self.status = Status::Won;
            } else {
                self.set_message("Terkunci. Cari kunci dulu.");
            }
            return;
        }

        if self.player.pos.distance(self.world.object_pos) < 2.0 {
            self.items.carrying = true;
            self.set_message("Kotak diambil.");
        }
    }

    pub fn prompt(&self) -> Option<&'static str> {
        if self.status != Status::Playing {
            return None;
        }
        if self.items.carrying {
            return Some("E: lepas kotak");
        }
        if !self.items.key_taken && self.player.pos.distance(self.world.key_pos) < 2.0 {
            return Some("E: ambil kunci");
        }
        if self.distance_to_door() < 2.4 {
            return Some(if self.items.has_key {
                "E: buka pintu"
            } else {
                "E: coba pintu (terkunci)"
            });
        }
        if self.player.pos.distance(self.world.object_pos) < 2.0 {
            return Some("E: ambil kotak");
        }
        None
    }

    pub fn alert_label(&self) -> Option<(&'static str, Color)> {
        match self.neighbor.state {
            AlertState::Chase => Some(("TERDETEKSI", Color::new(0.82, 0.22, 0.18, 1.0))),
            AlertState::Suspicious => Some(("MENCURIGAI", Color::new(0.85, 0.62, 0.18, 1.0))),
            AlertState::Patrol => None,
        }
    }

    pub fn draw(&self) {
        self.draw_scene(None);
        set_default_camera();
        ui::draw(self, screen_width(), screen_height());
    }

    pub fn draw_scene(&self, render_target: Option<RenderTarget>) {
        let eye = self.player.eye();
        let aspect = render_target.as_ref().map(|target| target.texture.width() / target.texture.height());
        let camera = Camera3D {
            position: eye,
            target: eye + self.player.forward(),
            up: vec3(0.0, 1.0, 0.0),
            fovy: 70.0_f32.to_radians(),
            aspect,
            render_target,
            ..Default::default()
        };
        set_camera(&camera);
        clear_background(Color::new(0.55, 0.66, 0.72, 1.0));

        for solid in &self.world.solids {
            draw_cube(solid.aabb.center(), solid.aabb.size(), None, solid.color);
        }

        if !self.world.door_open {
            draw_cube(
                self.world.door_center,
                self.world.door_size,
                None,
                Color::new(0.34, 0.22, 0.15, 1.0),
            );
        }

        if !self.items.key_taken {
            let key = self.world.key_pos;
            draw_cube(key, vec3(0.12, 0.06, 0.42), None, Color::new(0.85, 0.70, 0.25, 1.0));
            draw_cube(
                key + vec3(0.18, 0.0, 0.16),
                vec3(0.24, 0.06, 0.10),
                None,
                Color::new(0.85, 0.70, 0.25, 1.0),
            );
        }

        let object_pos = if self.items.carrying {
            self.player.eye() + self.player.forward() * 1.1 - vec3(0.0, 0.45, 0.0)
        } else {
            self.world.object_pos
        };
        draw_cube(
            object_pos,
            vec3(0.6, 0.6, 0.6),
            None,
            Color::new(0.62, 0.45, 0.26, 1.0),
        );

        let body = Color::new(0.24, 0.27, 0.32, 1.0);
        draw_cube(
            self.neighbor.pos + vec3(0.0, 0.9, 0.0),
            vec3(0.75, 1.8, 0.5),
            None,
            body,
        );
        draw_cube(
            self.neighbor.pos + vec3(0.0, 1.95, 0.0),
            vec3(0.45, 0.45, 0.45),
            None,
            Color::new(0.78, 0.62, 0.48, 1.0),
        );
    }

    fn distance_to_door(&self) -> f32 {
        let to_door = vec2(
            self.player.pos.x - self.world.door_center.x,
            self.player.pos.z - self.world.door_center.z,
        );
        to_door.length()
    }

    fn set_message(&mut self, text: &str) {
        self.message = text.to_string();
        self.message_timer = 2.5;
    }
}

fn axis(positive: KeyCode, negative: KeyCode) -> f32 {
    let mut value = 0.0;
    if is_key_down(positive) {
        value += 1.0;
    }
    if is_key_down(negative) {
        value -= 1.0;
    }
    value
}
