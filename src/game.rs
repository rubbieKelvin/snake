use std::collections::VecDeque;
use std::fs;

use sdl2::rect::Point;

use crate::constants::*;
use crate::objs::*;

pub struct Game {
    pub state: GameState,
    pub snake: VecDeque<Point>,
    pub dir: Direction,
    /// turns waiting to be applied, one per step, so fast key mashing can't reverse the snake
    queue: Vec<Direction>,
    pub eggs: Vec<Collectible>,
    pub viruses: Vec<Collectible>,
    pub score: u32,
    pub high_score: u32,
    pub new_high_score: bool,
    pub lives: u8,
    pub level: u32,
    pending_growth: usize,
    step_timer: f64,
    /// time remaining in the "ouch" flash
    pub damage_flash: f64,
}

impl Game {
    pub fn new() -> Self {
        let mut game = Game {
            state: GameState::Menu,
            snake: VecDeque::new(),
            dir: Direction::Right,
            queue: Vec::new(),
            eggs: Vec::new(),
            viruses: Vec::new(),
            score: 0,
            high_score: load_high_score(),
            new_high_score: false,
            lives: START_LIVES,
            level: 1,
            pending_growth: 0,
            step_timer: 0.0,
            damage_flash: 0.0,
        };
        game.reset();
        return game;
    }

    /// Starts a fresh round (keeps the high score).
    pub fn reset(&mut self) {
        let (cx, cy) = (COLS / 2, ROWS / 2);
        self.snake = (0..3).map(|i| Point::new(cx - i, cy)).collect();
        self.dir = Direction::Right;
        self.queue.clear();
        self.score = 0;
        self.new_high_score = false;
        self.lives = START_LIVES;
        self.level = 1;
        self.pending_growth = 0;
        self.step_timer = 0.0;
        self.damage_flash = 0.0;
        self.eggs.clear();
        self.viruses.clear();

        let pos = self.free_cell(0);
        self.eggs.push(Collectible::new(
            pos,
            CollectibleType::Egg { special: false },
        ));
        self.spawn_viruses();
    }

    pub fn start(&mut self) {
        self.reset();
        self.state = GameState::Playing;
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            other => other,
        };
    }

    pub fn turn(&mut self, wanted: Direction) {
        if self.state != GameState::Playing {
            return;
        }
        let last = self.queue.last().copied().unwrap_or(self.dir);
        if wanted == last || wanted == last.opposite() || self.queue.len() >= 2 {
            return;
        }
        self.queue.push(wanted);
    }

    pub fn step_interval(&self) -> f64 {
        (BASE_STEP_INTERVAL - STEP_SPEEDUP_PER_LEVEL * (self.level - 1) as f64)
            .max(MIN_STEP_INTERVAL)
    }

    pub fn update(&mut self, delta: f64) {
        self.damage_flash = (self.damage_flash - delta).max(0.0);
        if self.state != GameState::Playing {
            return;
        }

        for egg in self.eggs.iter_mut() {
            egg.age += delta;
        }
        self.expire_special_eggs();

        self.step_timer += delta;
        let interval = self.step_interval();
        while self.step_timer >= interval && self.state == GameState::Playing {
            self.step_timer -= interval;
            self.step();
        }
    }

    fn expire_special_eggs(&mut self) {
        for i in 0..self.eggs.len() {
            let expired = matches!(self.eggs[i].class, CollectibleType::Egg { special: true })
                && self.eggs[i].age >= SPECIAL_EGG_LIFETIME;
            if expired {
                self.respawn_egg(i);
            }
        }
    }

    fn step(&mut self) {
        if let Some(next) = self.queue.first().copied() {
            self.queue.remove(0);
            self.dir = next;
        }

        let new_head = step(self.snake[0], self.dir);

        // hitting yourself ends the game; the tail cell is free if it's about to move away
        let checked = self.snake.len() - if self.pending_growth == 0 { 1 } else { 0 };
        if self.snake.iter().take(checked).any(|c| *c == new_head) {
            self.game_over();
            return;
        }

        self.snake.push_front(new_head);
        if self.pending_growth > 0 {
            self.pending_growth -= 1;
        } else {
            self.snake.pop_back();
        }

        // virus: lose a life and part of your tail
        if let Some(i) = self.viruses.iter().position(|v| v.position == new_head) {
            self.viruses.remove(i);
            self.lives -= 1;
            self.damage_flash = DAMAGE_FLASH_TIME;
            let keep = self.snake.len().saturating_sub(VIRUS_TAIL_LOSS).max(2);
            self.snake.truncate(keep);
            self.pending_growth = 0;
            if self.lives == 0 {
                self.game_over();
            }
            return;
        }

        // egg
        if let Some(i) = self.eggs.iter().position(|e| e.position == new_head) {
            let credit = match self.eggs[i].class {
                CollectibleType::Egg { special: true } => 3,
                _ => 1,
            };
            self.score += credit;
            self.pending_growth += credit as usize;
            self.respawn_egg(i);

            let level = self.score / SCORE_PER_LEVEL + 1;
            if level > self.level {
                self.level = level;
                self.spawn_viruses();
                // a second egg to chase every few levels
                if self.level % 3 == 0 && self.eggs.len() < 3 {
                    let pos = self.free_cell(0);
                    self.eggs.push(Collectible::new(
                        pos,
                        CollectibleType::Egg { special: false },
                    ));
                }
            }
        }
    }

    fn game_over(&mut self) {
        self.state = GameState::GameOver;
        if self.score > self.high_score {
            self.high_score = self.score;
            self.new_high_score = true;
            save_high_score(self.high_score);
        }
    }

    fn respawn_egg(&mut self, index: usize) {
        let pos = self.free_cell(0);
        self.eggs[index] = Collectible::new(
            pos,
            CollectibleType::Egg {
                special: rand::random_bool(0.3),
            },
        );
    }

    fn spawn_viruses(&mut self) {
        let count = (self.level as usize + 2).min(12);
        self.viruses.clear();
        for _ in 0..count {
            // keep viruses clear of the head so a respawn can't be unfair
            let pos = self.free_cell(6);
            self.viruses
                .push(Collectible::new(pos, CollectibleType::Virus));
        }
    }

    /// A random cell not occupied by anything, at least `head_margin` cells from the head.
    fn free_cell(&self, head_margin: i32) -> Point {
        let head = self.snake[0];
        loop {
            let p = Point::new(rand::random_range(0..COLS), rand::random_range(0..ROWS));
            let far_enough = (p.x - head.x).abs() + (p.y - head.y).abs() > head_margin;
            let taken = self.snake.contains(&p)
                || self.eggs.iter().any(|e| e.position == p)
                || self.viruses.iter().any(|v| v.position == p);
            if far_enough && !taken {
                return p;
            }
        }
    }
}

fn load_high_score() -> u32 {
    return fs::read_to_string(HIGH_SCORE_FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0);
}

fn save_high_score(score: u32) {
    // best effort: failing to save shouldn't crash the game
    let _ = fs::write(HIGH_SCORE_FILE, score.to_string());
}
