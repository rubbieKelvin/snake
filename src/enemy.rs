use std::collections::VecDeque;

use sdl2::rect::Point;

use crate::objs::Direction;

pub struct Enemy {
    pub body: VecDeque<Point>,
    pub dir: Direction,
    pub timer: f64,
    /// index into the render palette
    pub color_idx: usize,
    /// hunters try to cut the player off instead of going for eggs
    pub hunter: bool,
    pub pending_growth: usize,
    pub alive: bool,
}

impl Enemy {
    pub fn new(body: VecDeque<Point>, dir: Direction, color_idx: usize, hunter: bool) -> Self {
        return Enemy {
            body,
            dir,
            timer: 0.0,
            color_idx,
            hunter,
            pending_growth: 0,
            alive: true,
        };
    }

    pub fn head(&self) -> Point {
        return self.body[0];
    }
}
