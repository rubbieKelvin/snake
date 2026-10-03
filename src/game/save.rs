use std::collections::VecDeque;
use std::fs;

use sdl2::rect::Point;
use serde::{Deserialize, Serialize};

use super::*;

const SAVE_FILE: &str = "savegame.json";

type Pt = (i32, i32);

#[derive(Serialize, Deserialize)]
struct ItemSave {
    pos: Pt,
    class: CollectibleType,
    age: f64,
}

#[derive(Serialize, Deserialize)]
struct EnemySave {
    body: Vec<Pt>,
    dir: Direction,
    timer: f64,
    color_idx: usize,
    hunter: bool,
    pending_growth: usize,
}

/// Everything needed to resume a round exactly where it stopped.
#[derive(Serialize, Deserialize)]
pub struct SaveData {
    cols: i32,
    rows: i32,
    snake: Vec<Pt>,
    dir: Direction,
    queue: Vec<Direction>,
    eggs: Vec<ItemSave>,
    viruses: Vec<ItemSave>,
    powers: Vec<ItemSave>,
    hearts: Vec<ItemSave>,
    enemies: Vec<EnemySave>,
    score: u32,
    lives: u8,
    level: u32,
    pending_growth: usize,
    step_timer: f64,
    damage_flash: f64,
    boost_meter: f64,
    boost_exhausted: bool,
    shield: bool,
    ghost: f64,
    freeze: f64,
    double: f64,
    power_timer: f64,
    enemy_spawn_timer: f64,
    enemies_spawned: usize,
}

fn pt(p: Point) -> Pt {
    return (p.x, p.y);
}

fn point(p: Pt) -> Point {
    return Point::new(p.0, p.1);
}

fn items_to_save(items: &[Collectible]) -> Vec<ItemSave> {
    return items
        .iter()
        .map(|c| ItemSave {
            pos: pt(c.position),
            class: c.class.clone(),
            age: c.age,
        })
        .collect();
}

fn items_from_save(items: &[ItemSave]) -> Vec<Collectible> {
    return items
        .iter()
        .map(|i| {
            let mut c = Collectible::new(point(i.pos), i.class.clone());
            c.age = i.age;
            c
        })
        .collect();
}

impl SaveData {
    /// Rejects saves from a different board size or with cells outside the board
    /// (a screen change, or a hand-edited / corrupt file).
    fn is_valid_for(&self, grid: Grid) -> bool {
        let inside = |p: &Pt| p.0 >= 0 && p.0 < grid.cols && p.1 >= 0 && p.1 < grid.rows;
        return self.cols == grid.cols
            && self.rows == grid.rows
            && self.snake.len() >= 2
            && self.snake.iter().all(inside)
            && self
                .enemies
                .iter()
                .all(|e| !e.body.is_empty() && e.body.iter().all(inside))
            && [&self.eggs, &self.viruses, &self.powers, &self.hearts]
                .iter()
                .all(|list| list.iter().all(|i| inside(&i.pos)))
            && self.lives >= 1
            && self.level >= 1;
    }
}

pub fn load_from_disk(grid: Grid) -> Option<SaveData> {
    if cfg!(test) {
        return None;
    }
    let text = fs::read_to_string(SAVE_FILE).ok()?;
    let data: SaveData = serde_json::from_str(&text).ok()?;
    return if data.is_valid_for(grid) {
        Some(data)
    } else {
        None
    };
}

pub fn delete_save() {
    if cfg!(not(test)) {
        let _ = fs::remove_file(SAVE_FILE);
    }
}

impl Game {
    /// (score, level) of the saved round, if there is one to continue.
    pub fn save_summary(&self) -> Option<(u32, u32)> {
        return self.saved.as_ref().map(|s| (s.score, s.level));
    }

    /// Writes the current round to disk. Only a round in progress is worth keeping.
    pub fn save_to_disk(&self) {
        if !matches!(self.state, GameState::Playing | GameState::Paused) || cfg!(test) {
            return;
        }
        // best effort: failing to save shouldn't crash the game
        if let Ok(text) = serde_json::to_string(&self.snapshot()) {
            let _ = fs::write(SAVE_FILE, text);
        }
    }

    /// Resumes the saved round, paused so the player can get ready.
    pub fn continue_game(&mut self) {
        if let Some(data) = self.saved.take() {
            self.restore(data);
            self.state = GameState::Paused;
            self.sfx(Sfx::Start);
        }
    }

    fn snapshot(&self) -> SaveData {
        return SaveData {
            cols: self.grid.cols,
            rows: self.grid.rows,
            snake: self.snake.iter().map(|p| pt(*p)).collect(),
            dir: self.dir,
            queue: self.queue.clone(),
            eggs: items_to_save(&self.eggs),
            viruses: items_to_save(&self.viruses),
            powers: items_to_save(&self.powers),
            hearts: items_to_save(&self.hearts),
            enemies: self
                .enemies
                .iter()
                .map(|e| EnemySave {
                    body: e.body.iter().map(|p| pt(*p)).collect(),
                    dir: e.dir,
                    timer: e.timer,
                    color_idx: e.color_idx,
                    hunter: e.hunter,
                    pending_growth: e.pending_growth,
                })
                .collect(),
            score: self.score,
            lives: self.lives,
            level: self.level,
            pending_growth: self.pending_growth,
            step_timer: self.step_timer,
            damage_flash: self.damage_flash,
            boost_meter: self.boost_meter,
            boost_exhausted: self.boost_exhausted,
            shield: self.shield,
            ghost: self.ghost,
            freeze: self.freeze,
            double: self.double,
            power_timer: self.power_timer,
            enemy_spawn_timer: self.enemy_spawn_timer,
            enemies_spawned: self.enemies_spawned,
        };
    }

    fn restore(&mut self, data: SaveData) {
        self.snake = data.snake.iter().map(|p| point(*p)).collect();
        self.dir = data.dir;
        self.queue = data.queue;
        self.eggs = items_from_save(&data.eggs);
        self.viruses = items_from_save(&data.viruses);
        self.powers = items_from_save(&data.powers);
        self.hearts = items_from_save(&data.hearts);
        self.enemies = data
            .enemies
            .into_iter()
            .map(|e| {
                let body: VecDeque<Point> = e.body.iter().map(|p| point(*p)).collect();
                let mut enemy = Enemy::new(body, e.dir, e.color_idx, e.hunter);
                enemy.timer = e.timer;
                enemy.pending_growth = e.pending_growth;
                enemy
            })
            .collect();
        self.popups.clear();
        self.sounds.clear();
        self.score = data.score;
        self.new_high_score = false;
        self.lives = data.lives;
        self.level = data.level;
        self.death_reason = "";
        self.pending_growth = data.pending_growth;
        self.step_timer = data.step_timer;
        self.damage_flash = data.damage_flash;
        self.boost_meter = data.boost_meter;
        self.boosting = false;
        self.boost_exhausted = data.boost_exhausted;
        self.shield = data.shield;
        self.ghost = data.ghost;
        self.freeze = data.freeze;
        self.double = data.double;
        self.power_timer = data.power_timer;
        self.enemy_spawn_timer = data.enemy_spawn_timer;
        self.enemies_spawned = data.enemies_spawned;
        self.over_age = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_round_trip_preserves_the_round() {
        let grid = Grid::for_screen(WINDOW_W, WINDOW_H);
        let mut game = Game::new(grid);
        game.start();
        for frame in 0..4000u32 {
            if game.state != GameState::Playing {
                game.start();
            }
            if frame % 9 == 0 {
                game.turn(Direction::ALL[rand::random_range(0..4)]);
            }
            if frame % 700 == 0 {
                game.add_score(10, Point::new(0, 0));
            }
            game.update(0.016);
            game.sounds.clear();
        }

        let json = serde_json::to_string(&game.snapshot()).unwrap();
        let data: SaveData = serde_json::from_str(&json).unwrap();
        assert!(data.is_valid_for(grid));

        let mut loaded = Game::new(grid);
        loaded.restore(data);
        assert_eq!(game.snake, loaded.snake);
        assert_eq!(game.score, loaded.score);
        assert_eq!(game.lives, loaded.lives);
        assert_eq!(game.level, loaded.level);
        assert_eq!(game.enemies.len(), loaded.enemies.len());
        assert_eq!(game.eggs.len(), loaded.eggs.len());
        assert_eq!(game.viruses.len(), loaded.viruses.len());
        assert_eq!(game.hearts.len(), loaded.hearts.len());

        // a save from another board size is rejected
        assert!(!json_for_other_grid(&game).is_valid_for(grid));
    }

    fn json_for_other_grid(game: &Game) -> SaveData {
        let mut data = game.snapshot();
        data.cols += 1;
        return data;
    }
}
