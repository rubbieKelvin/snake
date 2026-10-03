use crate::constants::*;
use sdl2::pixels::Color;
use sdl2::rect::Point;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub const ALL: [Direction; 4] = [
        Direction::Up,
        Direction::Down,
        Direction::Left,
        Direction::Right,
    ];

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
    return offset(from, dir.delta().0, dir.delta().1);
}

pub fn offset(from: Point, dx: i32, dy: i32) -> Point {
    return Point::new((from.x + dx).rem_euclid(COLS), (from.y + dy).rem_euclid(ROWS));
}

/// Manhattan distance on a board that wraps at the edges.
pub fn wrapped_dist(a: Point, b: Point) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    return dx.min(COLS - dx) + dy.min(ROWS - dy);
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PowerKind {
    Shield,
    Ghost,
    Freeze,
    Double,
}

impl PowerKind {
    pub const ALL: [PowerKind; 4] = [
        PowerKind::Shield,
        PowerKind::Ghost,
        PowerKind::Freeze,
        PowerKind::Double,
    ];

    pub fn name(self) -> &'static str {
        match self {
            PowerKind::Shield => "SHIELD",
            PowerKind::Ghost => "GHOST",
            PowerKind::Freeze => "FREEZE",
            PowerKind::Double => "2X SCORE",
        }
    }

    pub fn letter(self) -> &'static str {
        match self {
            PowerKind::Shield => "S",
            PowerKind::Ghost => "G",
            PowerKind::Freeze => "F",
            PowerKind::Double => "x2",
        }
    }

    pub fn describe(self) -> &'static str {
        match self {
            PowerKind::Shield => "absorbs the next hit",
            PowerKind::Ghost => "pass through your body and enemies",
            PowerKind::Freeze => "freezes all enemy snakes",
            PowerKind::Double => "double points",
        }
    }

    pub fn color(self) -> Color {
        match self {
            PowerKind::Shield => Color::RGB(70, 140, 255),
            PowerKind::Ghost => Color::RGB(200, 200, 235),
            PowerKind::Freeze => Color::RGB(150, 235, 255),
            PowerKind::Double => Color::RGB(255, 170, 30),
        }
    }
}

pub enum CollectibleType {
    Egg { special: bool },
    Virus,
    Power(PowerKind),
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

/// Floating text that rises from a grid cell (scores, pickups, warnings).
pub struct Popup {
    pub position: Point,
    pub text: String,
    pub color: Color,
    pub age: f64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Menu,
    Playing,
    Paused,
    GameOver,
}
