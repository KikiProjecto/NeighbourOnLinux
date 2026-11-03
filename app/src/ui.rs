use macroquad::prelude::*;

use crate::game::{GameState, Status};

pub fn draw(game: &GameState, width: f32, height: f32) {
    let center_x = width * 0.5;
    let center_y = height * 0.5;

    draw_line(center_x - 8.0, center_y, center_x - 3.0, center_y, 2.0, WHITE);
    draw_line(center_x + 3.0, center_y, center_x + 8.0, center_y, 2.0, WHITE);
    draw_line(center_x, center_y - 8.0, center_x, center_y - 3.0, 2.0, WHITE);
    draw_line(center_x, center_y + 3.0, center_x, center_y + 8.0, 2.0, WHITE);

    draw_text(
        "WASD gerak   E interaksi   Space lompat   Shift lari   Ctrl jongkok",
        16.0,
        26.0,
        18.0,
        Color::new(0.92, 0.92, 0.90, 1.0),
    );
    draw_text("R ulang setelah menang/kalah", 16.0, 48.0, 16.0, Color::new(0.80, 0.80, 0.78, 1.0));

    if let Some((label, color)) = game.alert_label() {
        let size = 30u16;
        let dimensions = measure_text(label, None, size, 1.0);
        draw_text(label, center_x - dimensions.width * 0.5, 64.0, size as f32, color);
    }

    if let Some(prompt) = game.prompt() {
        let size = 24u16;
        let dimensions = measure_text(prompt, None, size, 1.0);
        draw_text(
            prompt,
            center_x - dimensions.width * 0.5,
            height - 72.0,
            size as f32,
            WHITE,
        );
    }

    if game.message_timer > 0.0 && !game.message.is_empty() {
        draw_text(&game.message, 24.0, height - 28.0, 22.0, WHITE);
    }

    match game.status {
        Status::Playing => {}
        Status::Won => draw_overlay(width, height, "KAMU KABUR!", "R: main lagi"),
        Status::Lost => draw_overlay(width, height, "TERTANGKAP!", "R: coba lagi"),
    }
}

fn draw_overlay(width: f32, height: f32, title: &str, hint: &str) {
    draw_rectangle(0.0, 0.0, width, height, Color::new(0.0, 0.0, 0.0, 0.62));
    let title_size = 52u16;
    let title_dims = measure_text(title, None, title_size, 1.0);
    draw_text(
        title,
        width * 0.5 - title_dims.width * 0.5,
        height * 0.5,
        title_size as f32,
        WHITE,
    );
    let hint_size = 24u16;
    let hint_dims = measure_text(hint, None, hint_size, 1.0);
    draw_text(
        hint,
        width * 0.5 - hint_dims.width * 0.5,
        height * 0.5 + 44.0,
        hint_size as f32,
        Color::new(0.86, 0.86, 0.84, 1.0),
    );
}
