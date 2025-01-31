use std::time::Instant;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
};

use constants::*;
use objs::{Collectible, CollectibleType, SnakeCell, Timer, Vector2D};
use utils::{random_position_on_screen, render_text};

mod constants;
mod objs;
mod utils;

fn main() {
    #[allow(unused_variables)]
    let mut timer: f64 = 0.0;
    let mut score: u16 = 0;
    let mut snake_flash_count: u8 = 0; // when flashing we'd flash on every odd number, and reduce by one until zero

    let mut snake_cell_movement_timer = Timer::new(0.18);
    let mut egg_in_snake_body_timer = Timer::new(0.025);
    let mut snake_flash_timer = Timer::new(0.3);
    snake_flash_timer.pause();

    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", WINDOW_W, WINDOW_H)
        .position_centered()
        .build()
        .unwrap();

    let pixelify_font_28 = ttf_context
        .load_font("assets/fonts/pixelify.ttf", 28)
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // create a vector of snake cells at a randomly specified location
    let mut snake: Vec<SnakeCell> = vec![
        SnakeCell::new(Point::new(0, 0), Vector2D::new(1f32, 0f32)),
        SnakeCell::new(Point::new(0, 0), Vector2D::new(1f32, 0f32)),
    ];

    // egg
    let mut eggs: Vec<Collectible> = vec![Collectible {
        position: random_position_on_screen(),
        class: CollectibleType::Egg { special: false },
    }];

    let mut viruses = (1..10)
        .map(|_| Collectible {
            position: random_position_on_screen(),
            class: CollectibleType::Virus,
        })
        .collect::<Vec<Collectible>>();

    // let bounding_rect = Rect::new(0, 0, WINDOW_W, WINDOW_H);

    'running: loop {
        let start_time = Instant::now();

        // clear the canvas with the clear color
        canvas.set_draw_color(Color::RGB(30, 255, 50));
        canvas.clear();

        // check through the event poll
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => {
                    break 'running;
                }
                Event::KeyDown {
                    keycode: Some(code),
                    ..
                } => {
                    let cell_count = snake.len();
                    let cell = &mut snake[0];
                    match code {
                        Keycode::A | Keycode::LEFT => {
                            if cell_count > 1 && cell.direction.x == 1f32 {
                                // cannot co left if it's going right (this prevents it from directly turning on it's self)
                                // flash snake cells or show some kinda warning
                                snake_flash_count = MAX_SNAKE_FLASH_COUNT;
                                snake_flash_timer.play();
                            } else {
                                cell.direction = Vector2D::new(-1f32, 0f32);
                            }
                        }
                        Keycode::W | Keycode::UP => {
                            if cell_count > 1 && cell.direction.y == 1f32 {
                                // cannot co left if it's going right (this prevents it from directly turning on it's self)
                                // flash snake cells or show some kinda warning
                                snake_flash_count = MAX_SNAKE_FLASH_COUNT;
                                snake_flash_timer.play();
                            } else {
                                cell.direction = Vector2D::new(0f32, -1f32);
                            }
                        }
                        Keycode::D | Keycode::RIGHT => {
                            if cell_count > 1 && cell.direction.x == -1f32 {
                                snake_flash_count = MAX_SNAKE_FLASH_COUNT;
                                snake_flash_timer.play();
                            } else {
                                cell.direction = Vector2D::new(1f32, 0f32);
                            }
                        }
                        Keycode::S | Keycode::DOWN => {
                            if cell_count > 1 && cell.direction.y == -1f32 {
                                // cannot co left if it's going right (this prevents it from directly turning on it's self)
                                // flash snake cells or show some kinda warning
                                snake_flash_count = MAX_SNAKE_FLASH_COUNT;
                                snake_flash_timer.play();
                            } else {
                                cell.direction = Vector2D::new(0f32, 1f32);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // process data here...
        // see if we ate the egg
        {
            for egg in eggs.iter_mut() {
                let first_cell = &mut snake[0];

                let egg_rect = egg.rect();

                if first_cell.position.x == egg_rect.x && first_cell.position.y == egg_rect.y {
                    first_cell.just_swallowed_egg = true;

                    // increase score
                    let credit = if let CollectibleType::Egg { special: true } = egg.class {
                        3
                    } else {
                        1
                    };

                    score += credit;

                    // draw egg at another position
                    egg.position = random_position_on_screen();
                    egg.class = CollectibleType::Egg {
                        special: rand::random_bool(0.3),
                    };

                    // add cell to snake
                    let last_cell = &snake[snake.len() - 1];

                    // TODO: there's a bug here
                    // add a new cell to the snake at about the amount the user scored
                    snake.append(
                        &mut (0..credit)
                            .map(|i| {
                                // range is from 0 -> credit, so add one,
                                let index = i + 1;

                                SnakeCell::new(
                                    Point::new(
                                        last_cell.position.x
                                            - ((SNAKE_W as i32 * last_cell.direction.x as i32)
                                                * index as i32),
                                        last_cell.position.y
                                            - ((SNAKE_H as i32 * last_cell.direction.y as i32)
                                                * index as i32),
                                    ),
                                    Vector2D::new(last_cell.direction.x, last_cell.direction.y),
                                )
                            })
                            .collect::<Vec<SnakeCell>>(),
                    );
                }
            }
        }

        // process cell movement
        if snake_cell_movement_timer.triggered() {
            // move each cells direction to the one behind it
            // create a copy of the snake
            let snake_clone = snake
                .iter()
                .map(|cell| cell.copy())
                .collect::<Vec<SnakeCell>>();

            // set direction for cells
            for (index, cell) in snake.iter_mut().enumerate().rev() {
                // since we're iterating in reverse..
                if index == 0 {
                    break;
                };

                if let Some(copied_cell) = snake_clone.get(index - 1) {
                    cell.direction = copied_cell.direction.copy();
                }
            }

            // every time we get this tirgger,
            // move every cell's position by thier direction
            for cell in &mut snake {
                let mut new_x = cell.position.x + (cell.direction.x as i32 * SNAKE_W as i32);
                let mut new_y = cell.position.y + (cell.direction.y as i32 * SNAKE_H as i32);

                // allow teleporting on window border
                if new_x >= WINDOW_W as i32 {
                    new_x = 0;
                };
                if new_x < 0 {
                    new_x = WINDOW_W as i32;
                };

                if new_y >= WINDOW_H as i32 {
                    new_y = 0;
                }
                if new_y < 0 {
                    new_y = WINDOW_H as i32;
                }

                cell.position = Point::new(new_x, new_y);
            }
        }

        if egg_in_snake_body_timer.triggered() {
            // paint where the egg is at in the snakes body
            let mut holding_egg = false;
            for cell in snake.iter_mut() {
                if cell.just_swallowed_egg {
                    cell.just_swallowed_egg = false;
                    holding_egg = true;
                    continue;
                }

                if holding_egg {
                    cell.just_swallowed_egg = true;
                    holding_egg = false;
                }
            }
        }

        if snake_flash_timer.triggered() {
            snake_flash_count -= 1;
            if snake_flash_count == 0 {
                snake_flash_timer.stop();
            }
        }

        // then render ..

        // draw egg
        for egg in eggs.iter() {
            match egg.class {
                CollectibleType::Egg { special } => {
                    let egg_rect = egg.rect();
                    let shrink_factor = if special { 0 } else { 6 };

                    let visual_rect = Rect::new(
                        egg_rect.x + shrink_factor,
                        egg_rect.y + shrink_factor,
                        egg_rect.w as u32 - (shrink_factor as u32 * 2u32),
                        egg_rect.h as u32 - (shrink_factor as u32 * 2u32),
                    );
                    canvas.set_draw_color(if special { Color::YELLOW } else { Color::CYAN });
                    canvas.fill_rect(visual_rect).unwrap();
                    canvas.set_draw_color(Color::RGB(20, 20, 20));
                    canvas.draw_rect(visual_rect).unwrap();
                }
                CollectibleType::Virus => unreachable!(),
            }
        }

        // draw all cells at thier position
        let len = snake.len();
        for (index, cell) in snake.iter().enumerate() {
            let rect = Rect::new(cell.position.x, cell.position.y, SNAKE_W, SNAKE_H);

            let color = if cell.just_swallowed_egg {
                Color::CYAN
            } else {
                let ratio = (index as f32 + 1f32) / (len as f32);
                let iratio = ((len - index) as f32 + 1f32) / (len as f32);
                Color::RGB((255f32 * ratio) as u8, 100, (160f32 * iratio) as u8)
            };

            canvas.set_draw_color(color);
            canvas.fill_rect(rect).unwrap();

            // if we're flashing on an odd number, draw boarder
            if snake_flash_count % 2 == 1 {
                canvas.set_draw_color(Color::BLACK);
                canvas.draw_rect(rect).unwrap();
            }
        }

        // render score
        render_text(
            score.to_string().as_str(),
            Point::new(20, 20),
            &pixelify_font_28,
            &mut canvas,
            Color::BLACK,
        )
        .unwrap();

        // present the buffer on the window
        canvas.present();

        // time gone and shi
        let delta = Instant::now().duration_since(start_time).as_secs_f64();
        timer += delta;

        // tick clocks
        snake_cell_movement_timer.tick(delta);
        egg_in_snake_body_timer.tick(delta);
        snake_flash_timer.tick(delta);
    }
}
