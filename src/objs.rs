use crate::constants::*;
use sdl2::rect::Point;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    pub fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// Grid coordinates (not pixels), wrapped around the board edges.
pub fn step(from: Point, dir: Direction) -> Point {
    let (dx, dy) = dir.delta();
    return Point::new(
        (from.x + dx).rem_euclid(COLS),
        (from.y + dy).rem_euclid(ROWS),
    );
}

pub enum CollectibleType {
    Egg { special: bool },
    Virus,
}

pub struct Collectible {
    pub position: Point,
    pub class: CollectibleType,
    /// seconds since it appeared
    pub age: f64,
}

impl Collectible {
    pub fn new(position: Point, class: CollectibleType) -> Self {
        return Collectible {
            position,
            class,
            age: 0.0,
        };
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}
