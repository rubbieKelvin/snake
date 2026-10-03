use std::time::{Duration, Instant};

use sdl2::{
    event::Event,
    keyboard::{Keycode, Scancode},
};

use audio::Audio;
use constants::*;
use game::Game;
use objs::{Direction, GameState, Grid};
use render::{render, Fonts};

mod audio;
mod constants;
mod enemy;
mod game;
mod objs;
mod render;
mod utils;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", WINDOW_W, WINDOW_H)
        .position_centered()
        .fullscreen_desktop()
        .build()
        .unwrap();
    sdl_context.mouse().show_cursor(false);

    let load = |size: u16| {
        ttf_context
            .load_font("assets/fonts/pixelify.ttf", size)
            .unwrap()
    };
    let fonts = Fonts {
        tiny: load(14),
        small: load(20),
        normal: load(28),
        big: load(72),
    };

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // audio is optional: without a device the game simply runs silent
    let mut audio = sdl_context.audio().ok().and_then(|a| Audio::new(&a));

    // cells keep their size; a bigger screen just gives a bigger board
    let (screen_w, screen_h) = canvas.window().size();
    let mut game = Game::new(Grid::for_screen(screen_w, screen_h));
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
                    Keycode::C => {
                        if game.state == GameState::Menu {
                            game.continue_game();
                        }
                    }
                    Keycode::M => {
                        if let Some(audio) = audio.as_mut() {
                            audio.muted = !audio.muted;
                        }
                    }
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

        // boost is a held key, so poll it rather than reacting to events
        let keys = event_pump.keyboard_state();
        game.set_boost(
            keys.is_scancode_pressed(Scancode::Space)
                || keys.is_scancode_pressed(Scancode::LShift)
                || keys.is_scancode_pressed(Scancode::RShift),
        );

        let now = Instant::now();
        // clamp so a window drag or hiccup doesn't make the snake lurch
        let delta = now.duration_since(last_frame).as_secs_f64().min(0.1);
        last_frame = now;
        game.update(delta);

        for sfx in game.sounds.drain(..) {
            if let Some(audio) = audio.as_mut() {
                audio.play(sfx);
            }
        }

        render(&mut canvas, &game, &fonts);
        canvas.present();

        // vsync normally paces us; this covers drivers where it's unavailable
        if now.elapsed() < Duration::from_millis(2) {
            std::thread::sleep(Duration::from_millis(8));
        }
    }

    // quitting mid-round keeps it, so it can be continued next time
    game.save_to_disk();
}
