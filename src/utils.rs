use sdl2::rect::{Point, Rect};

use crate::constants::*;

pub fn random_position_on_screen() -> Point {
    let x = rand::random_range(0..(WINDOW_W / SNAKE_W) as i32);
    let y = rand::random_range(0..(WINDOW_H / SNAKE_H) as i32);
    return Point::new(x * SNAKE_W as i32, y * SNAKE_H as i32);
}

pub fn rect_from_point(point: &Point, w: u32, h: u32) -> Rect {
    return Rect::new(point.x, point.y, w, h);
}
