use std::time::{Duration, Instant};

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::WindowCanvas,
    ttf::Font,
};

use constants::*;
use game::Game;
use objs::{CollectibleType, Direction, GameState};
use utils::{render_text, render_text_centered};

mod constants;
mod game;
mod objs;
mod utils;

fn cell_rect(p: Point) -> Rect {
    return Rect::new(p.x * CELL as i32, p.y * CELL as i32, CELL, CELL);
}

fn inset(r: Rect, by: i32) -> Rect {
    return Rect::new(
        r.x + by,
        r.y + by,
        r.w as u32 - 2 * by as u32,
        r.h as u32 - 2 * by as u32,
    );
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", WINDOW_W, WINDOW_H)
        .position_centered()
        .build()
        .unwrap();

    let font_small = ttf_context
        .load_font("assets/fonts/pixelify.ttf", 22)
        .unwrap();
    let font = ttf_context
        .load_font("assets/fonts/pixelify.ttf", 28)
        .unwrap();
    let font_big = ttf_context
        .load_font("assets/fonts/pixelify.ttf", 72)
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut game = Game::new();
    let mut last_frame = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown {
                    keycode: Some(code),
                    repeat: false,
                    ..
                } => match code {
                    Keycode::Escape => break 'running,
                    Keycode::A | Keycode::Left => game.turn(Direction::Left),
                    Keycode::D | Keycode::Right => game.turn(Direction::Right),
                    Keycode::W | Keycode::Up => game.turn(Direction::Up),
                    Keycode::S | Keycode::Down => game.turn(Direction::Down),
                    Keycode::P => game.toggle_pause(),
                    Keycode::Return | Keycode::Space => match game.state {
                        GameState::Menu | GameState::GameOver => game.start(),
                        GameState::Paused => game.toggle_pause(),
                        GameState::Playing => {}
                    },
                    _ => {}
                },
                _ => {}
            }
        }

        let now = Instant::now();
        // clamp so a window drag or hiccup doesn't make the snake lurch
        let delta = now.duration_since(last_frame).as_secs_f64().min(0.1);
        last_frame = now;
        game.update(delta);

        render(&mut canvas, &game, &font_small, &font, &font_big);
        canvas.present();

        // vsync normally paces us; this covers drivers where it's unavailable
        if now.elapsed() < Duration::from_millis(2) {
            std::thread::sleep(Duration::from_millis(8));
        }
    }
}

fn render(canvas: &mut WindowCanvas, game: &Game, font_small: &Font, font: &Font, font_big: &Font) {
    // checkerboard background
    for y in 0..ROWS {
        for x in 0..COLS {
            canvas.set_draw_color(if (x + y) % 2 == 0 {
                Color::RGB(30, 220, 60)
            } else {
                Color::RGB(40, 235, 75)
            });
            canvas.fill_rect(cell_rect(Point::new(x, y))).unwrap();
        }
    }

    for egg in &game.eggs {
        if let CollectibleType::Egg { special } = egg.class {
            let rect = cell_rect(egg.position);
            // special eggs blink while about to expire
            let urgent = special && egg.age > SPECIAL_EGG_LIFETIME - 2.0;
            if urgent && (egg.age * 6.0) as i32 % 2 == 0 {
                continue;
            }
            let visual = if special { rect } else { inset(rect, 6) };
            canvas.set_draw_color(if special { Color::YELLOW } else { Color::CYAN });
            canvas.fill_rect(visual).unwrap();
            canvas.set_draw_color(Color::RGB(20, 20, 20));
            canvas.draw_rect(visual).unwrap();
        }
    }

    for virus in &game.viruses {
        let rect = cell_rect(virus.position);
        canvas.set_draw_color(Color::RGB(150, 0, 170));
        canvas.fill_rect(inset(rect, 3)).unwrap();
        canvas.set_draw_color(Color::RGB(230, 60, 90));
        canvas.fill_rect(inset(rect, 7)).unwrap();
        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas.draw_rect(inset(rect, 3)).unwrap();
    }

    // snake
    let len = game.snake.len();
    let hurt_flash = game.damage_flash > 0.0 && (game.damage_flash * 10.0) as i32 % 2 == 0;
    let dead = game.state == GameState::GameOver;
    for (index, cell) in game.snake.iter().enumerate() {
        let rect = cell_rect(*cell);
        let ratio = (index as f32 + 1.0) / len as f32;
        let iratio = ((len - index) as f32 + 1.0) / len as f32;
        let color = if hurt_flash {
            Color::WHITE
        } else if dead {
            Color::RGB(110, 110, 110)
        } else {
            Color::RGB(
                (255.0 * ratio) as u8,
                100,
                (160.0 * iratio).min(255.0) as u8,
            )
        };
        canvas.set_draw_color(color);
        canvas.fill_rect(rect).unwrap();
    }

    // eyes on the head
    if let Some(head) = game.snake.front() {
        let r = cell_rect(*head);
        let (dx, dy) = game.dir.delta();
        // two eyes offset perpendicular to the heading, pushed forward a bit
        let (px, py) = (dy.abs(), dx.abs());
        for side in [-1, 1] {
            let ex = r.x + 8 + dx * 4 + px * side * 5;
            let ey = r.y + 8 + dy * 4 + py * side * 5;
            canvas.set_draw_color(Color::WHITE);
            canvas.fill_rect(Rect::new(ex, ey, 4, 4)).unwrap();
        }
    }

    // HUD
    let ink = Color::RGB(15, 40, 20);
    render_text(
        &format!("Score {}", game.score),
        Point::new(20, 14),
        font,
        canvas,
        ink,
    )
    .unwrap();
    render_text(
        &format!("Level {}   Best {}", game.level, game.high_score),
        Point::new(20, 50),
        font_small,
        canvas,
        ink,
    )
    .unwrap();
    for i in 0..game.lives as i32 {
        canvas.set_draw_color(Color::RGB(230, 40, 70));
        canvas
            .fill_rect(Rect::new(WINDOW_W as i32 - 40 - i * 32, 22, 22, 22))
            .unwrap();
        canvas.set_draw_color(Color::RGB(20, 20, 20));
        canvas
            .draw_rect(Rect::new(WINDOW_W as i32 - 40 - i * 32, 22, 22, 22))
            .unwrap();
    }

    let cx = WINDOW_W as i32 / 2;
    let white = Color::WHITE;
    match game.state {
        GameState::Playing => {}
        state => {
            canvas.set_blend_mode(sdl2::render::BlendMode::Blend);
            canvas.set_draw_color(Color::RGBA(0, 0, 0, 150));
            canvas
                .fill_rect(Rect::new(0, 0, WINDOW_W, WINDOW_H))
                .unwrap();

            match state {
                GameState::Menu => {
                    render_text_centered("SNAKE", cx, 200, font_big, canvas, white).unwrap();
                    render_text_centered("Press ENTER to start", cx, 330, font, canvas, white)
                        .unwrap();
                    render_text_centered(
                        "Eat eggs. Cyan +1, yellow +3 (they vanish!)",
                        cx,
                        410,
                        font_small,
                        canvas,
                        white,
                    )
                    .unwrap();
                    render_text_centered(
                        "Avoid the red viruses: each costs a life and your tail.",
                        cx,
                        445,
                        font_small,
                        canvas,
                        white,
                    )
                    .unwrap();
                    render_text_centered(
                        "Don't bite yourself. Edges wrap around. Speed rises each level.",
                        cx,
                        480,
                        font_small,
                        canvas,
                        white,
                    )
                    .unwrap();
                    render_text_centered(
                        "WASD / arrows to move   P pause   ESC quit",
                        cx,
                        540,
                        font_small,
                        canvas,
                        white,
                    )
                    .unwrap();
                    if game.high_score > 0 {
                        render_text_centered(
                            &format!("Best: {}", game.high_score),
                            cx,
                            600,
                            font,
                            canvas,
                            Color::YELLOW,
                        )
                        .unwrap();
                    }
                }
                GameState::Paused => {
                    render_text_centered("PAUSED", cx, 300, font_big, canvas, white).unwrap();
                    render_text_centered(
                        "Press P or ENTER to resume",
                        cx,
                        400,
                        font,
                        canvas,
                        white,
                    )
                    .unwrap();
                }
                GameState::GameOver => {
                    render_text_centered("GAME OVER", cx, 240, font_big, canvas, white).unwrap();
                    render_text_centered(
                        &format!("Score {}   Level {}", game.score, game.level),
                        cx,
                        350,
                        font,
                        canvas,
                        white,
                    )
                    .unwrap();
                    let best = if game.new_high_score {
                        "New high score!".to_string()
                    } else {
                        format!("Best {}", game.high_score)
                    };
                    render_text_centered(&best, cx, 400, font, canvas, Color::YELLOW).unwrap();
                    render_text_centered("Press ENTER to play again", cx, 480, font, canvas, white)
                        .unwrap();
                }
                GameState::Playing => {}
            }
            canvas.set_blend_mode(sdl2::render::BlendMode::None);
        }
    }
}
