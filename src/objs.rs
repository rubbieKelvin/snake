use crate::constants::*;
use sdl2::pixels::Color;
use sdl2::rect::Point;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
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

/// The playfield: as many CELL-sized cells as fit below the header, so a bigger screen
/// means a bigger board rather than a bigger scale.
#[derive(Clone, Copy)]
pub struct Grid {
    pub cols: i32,
    pub rows: i32,
    /// full screen size in pixels
    pub width: u32,
    pub height: u32,
}

impl Grid {
    pub fn for_screen(width: u32, height: u32) -> Grid {
        return Grid {
            cols: ((width / CELL) as i32).max(20),
            rows: ((height as i32 - HUD_H) / CELL as i32).max(15),
            width,
            height,
        };
    }

    /// Pixel position of the board's top-left cell, centered in the space below the header.
    pub fn origin(&self) -> Point {
        let spare_x = self.width as i32 - self.cols * CELL as i32;
        let spare_y = self.height as i32 - HUD_H - self.rows * CELL as i32;
        return Point::new(spare_x / 2, HUD_H + spare_y / 2);
    }
}

/// Grid coordinates (not pixels), wrapped around the board edges.
pub fn step(grid: Grid, from: Point, dir: Direction) -> Point {
    return offset(grid, from, dir.delta().0, dir.delta().1);
}

pub fn offset(grid: Grid, from: Point, dx: i32, dy: i32) -> Point {
    return Point::new(
        (from.x + dx).rem_euclid(grid.cols),
        (from.y + dy).rem_euclid(grid.rows),
    );
}

/// Manhattan distance on a board that wraps at the edges.
pub fn wrapped_dist(grid: Grid, a: Point, b: Point) -> i32 {
    let dx = (a.x - b.x).abs();
    let dy = (a.y - b.y).abs();
    return dx.min(grid.cols - dx) + dy.min(grid.rows - dy);
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
pub enum CollectibleType {
    Egg {
        special: bool,
    },
    Virus,
    Power(PowerKind),
    /// restores a life; offered at the start of a level
    Heart,
    /// refills the chamber
    Ammo,
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

/// A round fired from the player's head; travels one cell per BULLET_STEP_INTERVAL.
pub struct Bullet {
    pub position: Point,
    pub dir: Direction,
    pub timer: f64,
    /// cells flown so far
    pub travelled: i32,
    /// hit something or ran out of range; removed at the end of the frame
    pub spent: bool,
}

impl Bullet {
    pub fn new(position: Point, dir: Direction) -> Self {
        return Bullet {
            position,
            dir,
            timer: 0.0,
            travelled: 0,
            spent: false,
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
