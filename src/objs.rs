use crate::constants::*;
use crate::utils::rect_from_point;
use sdl2::rect::{Point, Rect};

pub struct Vector2D {
    pub x: f32,
    pub y: f32,
}

impl Vector2D {
    pub fn new(x: f32, y: f32) -> Self {
        return Vector2D { x, y };
    }

    #[allow(unused)]
    fn zero() -> Self {
        return Vector2D { x: 0f32, y: 0f32 };
    }

    pub fn copy(&self) -> Self {
        return Vector2D::new(self.x, self.y);
    }
}

pub struct SnakeCell {
    pub position: Point,
    pub direction: Vector2D,
    // just a flag to tell if the swallowed egg is at this point in the cell
    // this flag is just for asthetics
    pub just_swallowed_egg: bool,
}

impl SnakeCell {
    pub fn new(pos: Point, direction: Vector2D) -> Self {
        return SnakeCell {
            position: pos,
            direction,
            just_swallowed_egg: false,
        };
    }

    pub fn copy(&self) -> Self {
        return SnakeCell {
            position: Point::new(self.position.x, self.position.y),
            direction: self.direction.copy(),
            just_swallowed_egg: self.just_swallowed_egg,
        };
    }
}

pub struct Collectible {
    pub position: Point,
    pub class: CollectibleType,
}

impl Collectible {
    pub fn rect(&self) -> Rect {
        return rect_from_point(&self.position, SNAKE_H, SNAKE_W);
    }
}

pub enum CollectibleType {
    Egg { special: bool },
    Virus,
}

pub struct Timer {
    running: bool,
    interval: f64,
    counter: f64,
}

impl Timer {
    pub fn new(interval: f64) -> Self {
        return Timer {
            running: true,
            interval,
            counter: 0.0,
        };
    }

    pub fn triggered(&mut self) -> bool {
        if self.counter >= self.interval {
            self.counter = 0.0;
            return true;
        }
        return false;
    }

    pub fn tick(&mut self, delta: f64) {
        if self.running {
            self.counter += delta;
        }
    }
}
