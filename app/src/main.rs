mod ai;
mod capture;
mod game;
mod items;
mod player;
mod ui;
mod world;

use macroquad::miniquad::conf::{LinuxBackend, Platform};
use macroquad::prelude::*;

struct Args {
    screenshot: Option<String>,
    self_test: bool,
    frames: Option<u32>,
}

fn main() {
    let args = parse_args();
    if args.self_test {
        capture::self_test();
        return;
    }
    macroquad::Window::from_config(window_conf(), run(args));
}

fn parse_args() -> Args {
    let mut args = Args {
        screenshot: None,
        self_test: false,
        frames: None,
    };
    let mut values = std::env::args().skip(1);
    while let Some(flag) = values.next() {
        match flag.as_str() {
            "--screenshot" => args.screenshot = values.next(),
            "--self-test" => args.self_test = true,
            "--frames" => args.frames = values.next().and_then(|value| value.parse().ok()),
            _ => eprintln!("unknown argument: {flag}"),
        }
    }
    args
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Hello Neighbor".to_string(),
        window_width: 1280,
        window_height: 720,
        window_resizable: true,
        platform: Platform {
            linux_backend: LinuxBackend::WaylandWithX11Fallback,
            ..Default::default()
        },
        ..Default::default()
    }
}

async fn run(args: Args) {
    set_cursor_grab(true);
    show_mouse(false);

    let mut game = game::GameState::new();
    let mut frames = 0u32;
    loop {
        let dt = get_frame_time().min(0.05);
        game.update(dt);
        game.draw();
        frames += 1;

        if let Some(path) = &args.screenshot {
            if frames >= 3 {
                capture::save_screenshot(path, &game);
                std::process::exit(0);
            }
        }
        next_frame().await;

        if let Some(limit) = args.frames {
            if frames >= limit {
                std::process::exit(0);
            }
        }
    }
}
