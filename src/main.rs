use std::time::Instant;

use sdl2::{
    event::Event,
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
};

const SNAKE_W: u32 = 20;
const SNAKE_H: u32 = 20;

const WINDOW_W: u32 = 1400;
const WINDOW_H: u32 = 800;

struct Vector2D {
    x: f32,
    y: f32,
}

impl Vector2D {
    fn new(x: f32, y: f32) -> Self {
        return Vector2D { x, y };
    }

    #[allow(unused)]
    fn zero() -> Self {
        return Vector2D { x: 0f32, y: 0f32 };
    }
}

struct SnakeCell {
    position: Point,
    direction: Vector2D,
}

impl SnakeCell {
    fn new(pos: Point, direction: Vector2D) -> Self {
        return SnakeCell {
            position: pos,
            direction,
        };
    }
}

struct Timer {
    running: bool,
    interval: f64,
    counter: f64,
}

impl Timer {
    fn new(interval: f64) -> Self {
        return Timer {
            running: true,
            interval,
            counter: 0.0,
        };
    }

    fn triggered(&mut self) -> bool {
        if self.counter >= self.interval {
            self.counter = 0.0;
            return true;
        }
        return false;
    }

    fn tick(&mut self, delta: f64) {
        if self.running {
            self.counter += delta;
        }
    }
}

fn random_position_on_screen() -> Point {
    let x = rand::random_range(0..(WINDOW_W / SNAKE_W) as i32);
    let y = rand::random_range(0..(WINDOW_H / SNAKE_H) as i32);
    return Point::new(x * SNAKE_W as i32, y * SNAKE_H as i32);
}

fn rect_from_point(point: &Point, w: u32, h: u32) -> Rect {
    return Rect::new(point.x, point.y, w, h);
}

fn main() {
    #[allow(unused_variables)]
    let mut timer: f64 = 0.0;
    let mut score: u16 = 0;

    let mut snake_cell_movement_timer = Timer::new(0.18);

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Snake game", WINDOW_W, WINDOW_H)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    // create a vector of snake cells at a randomly specified location
    let mut snake: Vec<SnakeCell> = vec![
        SnakeCell::new(Point::new(0, 0), Vector2D::new(1f32, 0f32)),
        SnakeCell::new(Point::new(0, 0), Vector2D::new(1f32, 0f32)),
    ];

    // egg
    let mut egg_rect = rect_from_point(&random_position_on_screen(), SNAKE_H, SNAKE_W);

    // let bounding_rect = Rect::new(0, 0, WINDOW_W, WINDOW_H);

    'running: loop {
        let start_time = Instant::now();

        // clear the canvas with the clear color
        canvas.set_draw_color(Color::BLACK);
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
                    let cell = &mut snake[0];
                    match code {
                        Keycode::A | Keycode::LEFT => cell.direction = Vector2D::new(-1f32, 0f32),
                        Keycode::W | Keycode::UP => cell.direction = Vector2D::new(0f32, -1f32),
                        Keycode::D | Keycode::RIGHT => cell.direction = Vector2D::new(1f32, 0f32),
                        Keycode::S | Keycode::DOWN => cell.direction = Vector2D::new(0f32, 1f32),
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // process data here...
        // see if we ate the egg
        {
            let first_cell = &snake[0];

            if first_cell.position.x == egg_rect.x && first_cell.position.y == egg_rect.y {
                // increase score
                score += 1;

                // draw egg at another position
                egg_rect = rect_from_point(&random_position_on_screen(), SNAKE_H, SNAKE_W);

                // add cell to snake
                let last_cell = &snake[snake.len() - 1];

                snake.push(SnakeCell::new(
                    Point::new(
                        last_cell.position.x - (SNAKE_W as i32 * last_cell.direction.x as i32),
                        last_cell.position.y - (SNAKE_H as i32 * last_cell.direction.y as i32),
                    ),
                    Vector2D::new(last_cell.direction.x, last_cell.direction.y),
                ));
            }
        }

        // process cell movement
        if snake_cell_movement_timer.triggered() {
            // move each cells direction to the one behind it
            // create a copy of the directions
            let directions_list = snake
                .iter()
                .map(|cell| Vector2D::new(cell.direction.x, cell.direction.y))
                .collect::<Vec<Vector2D>>();

            for (index, cell) in snake.iter_mut().enumerate().rev() {
                // since we're iterating in reverse..
                if index == 0 {
                    break;
                };

                if let Some(dir_at_the_front) = directions_list.get(index - 1) {
                    cell.direction = Vector2D::new(dir_at_the_front.x, dir_at_the_front.y);
                }
            }

            // every time we get this tirgger,
            // move every cell's position by thier direction
            for cell in &mut snake {
                let mut new_x = cell.position.x + (cell.direction.x as i32 * SNAKE_W as i32);
                let mut new_y = cell.position.y + (cell.direction.y as i32 * SNAKE_H as i32);

                // allow teleporting in window
                if new_x >= WINDOW_W as i32 {
                    new_x = 0
                };
                if new_x < 0 {
                    new_x = WINDOW_W as i32
                };

                cell.position = Point::new(new_x, new_y);
            }
        }

        // then render ..
        // draw bounding rect
        // canvas.set_draw_color(Color::RGB(50, 50, 50));
        // canvas.draw_rect(bounding_rect).unwrap();

        // draw egg
        canvas.set_draw_color(Color::YELLOW);
        canvas
            .fill_rect(Rect::new(
                egg_rect.x + 5,
                egg_rect.y + 5,
                egg_rect.w as u32 - 10,
                egg_rect.h as u32 - 10,
            ))
            .unwrap();

        // draw all cells at thier position
        let len = snake.len();
        for (index, cell) in snake.iter().enumerate() {
            let rect = Rect::new(cell.position.x, cell.position.y, SNAKE_W, SNAKE_H);

            let ratio = (index as f32 + 1f32) / (len as f32);
            let iratio = ((len - index) as f32 + 1f32) / (len as f32);
            let color = Color::RGB((255f32 * ratio) as u8, 100, (160f32 * iratio) as u8);

            canvas.set_draw_color(color);
            canvas.fill_rect(rect).unwrap();
        }

        // present the buffer on the window
        canvas.present();

        // time gone and shi
        let time_gone = Instant::now().duration_since(start_time).as_secs_f64();
        timer += time_gone;

        // tick clocks
        snake_cell_movement_timer.tick(time_gone);
    }
}
