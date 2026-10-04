use std::time::{Duration, Instant};

use sdl2::{
    controller::{Axis, Button, GameController},
    event::Event,
    keyboard::{Keycode, Scancode},
    GameControllerSubsystem,
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

    // gamepads are optional too: a controller plugged in later is picked up on the fly
    let controller_subsystem = sdl_context.game_controller().ok();
    let mut controller: Option<GameController> = controller_subsystem
        .as_ref()
        .and_then(open_first_controller);
    let mut last_stick: Option<Direction> = None;

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
                Event::ControllerDeviceAdded { which, .. } => {
                    if controller.is_none() {
                        controller = controller_subsystem
                            .as_ref()
                            .and_then(|sub| sub.open(which).ok());
                    }
                }
                Event::ControllerDeviceRemoved { which, .. } => {
                    if controller
                        .as_ref()
                        .is_some_and(|c| c.instance_id() == which)
                    {
                        controller = None;
                    }
                }
                Event::ControllerButtonDown { button, .. } => match button {
                    Button::DPadLeft => game.turn(Direction::Left),
                    Button::DPadRight => game.turn(Direction::Right),
                    Button::DPadUp => game.turn(Direction::Up),
                    Button::DPadDown => game.turn(Direction::Down),
                    // Cross (bottom face button) starts and resumes, like ENTER
                    Button::A => match game.state {
                        GameState::Menu | GameState::GameOver => game.start(),
                        GameState::Paused => game.toggle_pause(),
                        GameState::Playing => {}
                    },
                    // Start/Options pauses, or starts from a menu
                    Button::Start => match game.state {
                        GameState::Menu | GameState::GameOver => game.start(),
                        GameState::Playing | GameState::Paused => game.toggle_pause(),
                    },
                    // Triangle continues a saved round, like C
                    Button::Y => {
                        if game.state == GameState::Menu {
                            game.continue_game();
                        }
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        // boost is a held input, so poll the keyboard and the pad rather than
        // reacting to events
        let keys = event_pump.keyboard_state();
        let pad_boost = controller.as_ref().is_some_and(|c| {
            c.button(Button::B)
                || c.button(Button::RightShoulder)
                || c.axis(Axis::TriggerRight) > 12000
                || c.axis(Axis::TriggerLeft) > 12000
        });
        game.set_boost(
            keys.is_scancode_pressed(Scancode::Space)
                || keys.is_scancode_pressed(Scancode::LShift)
                || keys.is_scancode_pressed(Scancode::RShift)
                || pad_boost,
        );

        // left stick steering, edge-triggered so it can't spam the turn queue
        let stick = controller.as_ref().and_then(stick_direction);
        if stick != last_stick {
            if let Some(dir) = stick {
                game.turn(dir);
            }
            last_stick = stick;
        }

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

/// Opens the first connected pad, if any. Returns `None` when none is plugged in.
fn open_first_controller(subsystem: &GameControllerSubsystem) -> Option<GameController> {
    let count = subsystem.num_joysticks().ok()?;
    return (0..count).find_map(|i| {
        if subsystem.is_game_controller(i) {
            subsystem.open(i).ok()
        } else {
            None
        }
    });
}

/// Left stick direction, ignoring drift around center.
fn stick_direction(controller: &GameController) -> Option<Direction> {
    const DEAD_ZONE: i16 = 12000;
    let x = controller.axis(Axis::LeftX).saturating_abs();
    let y = controller.axis(Axis::LeftY).saturating_abs();
    if x < DEAD_ZONE && y < DEAD_ZONE {
        return None;
    }
    return Some(if x > y {
        if controller.axis(Axis::LeftX) > 0 {
            Direction::Right
        } else {
            Direction::Left
        }
    } else if controller.axis(Axis::LeftY) > 0 {
        Direction::Down
    } else {
        Direction::Up
    });
}
