use std::collections::VecDeque;
use std::fs;

use sdl2::pixels::Color;
use sdl2::rect::Point;

use crate::audio::Sfx;
use crate::constants::*;
use crate::enemy::Enemy;
use crate::objs::*;

pub struct Game {
    pub state: GameState,
    pub snake: VecDeque<Point>,
    pub dir: Direction,
    /// turns waiting to be applied, one per step, so fast key mashing can't reverse the snake
    queue: Vec<Direction>,
    pub eggs: Vec<Collectible>,
    pub viruses: Vec<Collectible>,
    pub powers: Vec<Collectible>,
    pub enemies: Vec<Enemy>,
    pub popups: Vec<Popup>,
    /// sound events waiting for the audio layer to play them
    pub sounds: Vec<Sfx>,
    pub score: u32,
    pub high_score: u32,
    pub new_high_score: bool,
    pub lives: u8,
    pub level: u32,
    pub death_reason: &'static str,
    pending_growth: usize,
    step_timer: f64,
    /// time remaining in the "ouch" flash
    /// and grants brief invulnerability
    pub damage_flash: f64,

    // boost
    pub boost_meter: f64,
    pub boosting: bool,
    pub boost_exhausted: bool,
    boost_held: bool,

    // active power-ups
    pub shield: bool,
    pub ghost: f64,
    pub freeze: f64,
    pub double: f64,

    power_timer: f64,
    enemy_spawn_timer: f64,
    enemies_spawned: usize,
    over_age: f64,
    /// seconds of real time, for animations
    pub clock: f64,
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
            powers: Vec::new(),
            enemies: Vec::new(),
            popups: Vec::new(),
            sounds: Vec::new(),
            score: 0,
            high_score: load_high_score(),
            new_high_score: false,
            lives: START_LIVES,
            level: 1,
            death_reason: "",
            pending_growth: 0,
            step_timer: 0.0,
            damage_flash: 0.0,
            boost_meter: 1.0,
            boosting: false,
            boost_exhausted: false,
            boost_held: false,
            shield: false,
            ghost: 0.0,
            freeze: 0.0,
            double: 0.0,
            power_timer: 0.0,
            enemy_spawn_timer: 0.0,
            enemies_spawned: 0,
            over_age: 0.0,
            clock: 0.0,
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
        self.death_reason = "";
        self.pending_growth = 0;
        self.step_timer = 0.0;
        self.damage_flash = 0.0;
        self.boost_meter = 1.0;
        self.boosting = false;
        self.boost_exhausted = false;
        self.shield = false;
        self.ghost = 0.0;
        self.freeze = 0.0;
        self.double = 0.0;
        self.power_timer = 0.0;
        self.enemy_spawn_timer = 0.0;
        self.enemies_spawned = 0;
        self.over_age = 0.0;
        self.eggs.clear();
        self.viruses.clear();
        self.powers.clear();
        self.enemies.clear();
        self.popups.clear();

        let pos = self.free_cell(0);
        self.eggs.push(Collectible::new(
            pos,
            CollectibleType::Egg { special: false },
        ));
        self.spawn_viruses();
    }

    pub fn start(&mut self) {
        // stops a key press made just before dying from instantly restarting
        if self.state == GameState::GameOver && self.over_age < RESTART_LOCKOUT {
            return;
        }
        self.reset();
        self.state = GameState::Playing;
        self.sfx(Sfx::Start);
    }

    pub fn toggle_pause(&mut self) {
        self.state = match self.state {
            GameState::Playing => GameState::Paused,
            GameState::Paused => GameState::Playing,
            other => other,
        };
    }

    pub fn set_boost(&mut self, held: bool) {
        self.boost_held = held;
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

    pub fn base_step_interval(&self) -> f64 {
        return (BASE_STEP_INTERVAL - STEP_SPEEDUP_PER_LEVEL * (self.level - 1) as f64)
            .max(MIN_STEP_INTERVAL);
    }

    pub fn step_interval(&self) -> f64 {
        let factor = if self.boosting {
            BOOST_SPEED_FACTOR
        } else {
            1.0
        };
        return self.base_step_interval() * factor;
    }

    fn enemy_interval(&self) -> f64 {
        return (self.base_step_interval() + 0.05).max(0.11);
    }

    fn enemy_target(&self) -> usize {
        return ((self.level / 2) as usize).min(MAX_ENEMIES);
    }

    pub fn update(&mut self, delta: f64) {
        self.clock += delta;
        self.damage_flash = (self.damage_flash - delta).max(0.0);
        for popup in self.popups.iter_mut() {
            popup.age += delta;
        }
        self.popups.retain(|p| p.age < POPUP_LIFETIME);

        match self.state {
            GameState::Playing => {}
            GameState::GameOver => {
                self.over_age += delta;
                return;
            }
            _ => return,
        }

        self.update_boost(delta);
        self.ghost = (self.ghost - delta).max(0.0);
        self.freeze = (self.freeze - delta).max(0.0);
        self.double = (self.double - delta).max(0.0);

        for item in self.eggs.iter_mut().chain(self.powers.iter_mut()) {
            item.age += delta;
        }
        self.powers.retain(|p| p.age < POWER_LIFETIME);
        self.expire_special_eggs();

        self.power_timer += delta;
        if self.power_timer >= POWER_SPAWN_INTERVAL {
            self.power_timer = 0.0;
            if self.powers.len() < 2 {
                let kind = PowerKind::ALL[rand::random_range(0..PowerKind::ALL.len())];
                let pos = self.free_cell(0);
                self.powers
                    .push(Collectible::new(pos, CollectibleType::Power(kind)));
            }
        }

        if self.enemies.len() < self.enemy_target() {
            self.enemy_spawn_timer += delta;
            if self.enemy_spawn_timer >= ENEMY_SPAWN_DELAY {
                self.enemy_spawn_timer = 0.0;
                self.try_spawn_enemy();
            }
        }

        self.step_timer += delta;
        let interval = self.step_interval();
        while self.step_timer >= interval && self.state == GameState::Playing {
            self.step_timer -= interval;
            self.step();
        }

        if self.state == GameState::Playing {
            self.step_enemies(delta);
        }
    }

    fn update_boost(&mut self, delta: f64) {
        let was_boosting = self.boosting;
        let wanted = self.boost_held && !self.boost_exhausted && self.boost_meter > 0.0;
        if wanted {
            self.boost_meter -= BOOST_DRAIN * delta;
            if self.boost_meter <= 0.0 {
                self.boost_meter = 0.0;
                // must refill a bit before boosting again, so it can't be held for free
                self.boost_exhausted = true;
            }
        } else {
            self.boost_meter = (self.boost_meter + BOOST_REGEN * delta).min(1.0);
            if self.boost_exhausted && self.boost_meter >= BOOST_RESUME {
                self.boost_exhausted = false;
            }
        }
        self.boosting = wanted && self.boost_meter > 0.0;
        if self.boosting && !was_boosting {
            self.sfx(Sfx::Boost);
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

    fn sfx(&mut self, sfx: Sfx) {
        if self.sounds.len() < 32 {
            self.sounds.push(sfx);
        }
    }

    fn popup(&mut self, at: Point, text: &str, color: Color) {
        self.popups.push(Popup {
            position: at,
            text: text.to_string(),
            color,
            age: 0.0,
        });
    }

    fn step(&mut self) {
        if let Some(next) = self.queue.first().copied() {
            self.queue.remove(0);
            self.dir = next;
        }

        let new_head = step(self.snake[0], self.dir);

        // hitting yourself ends the game; the tail cell is free if it's about to move away
        if self.ghost <= 0.0 {
            let checked = self.snake.len() - if self.pending_growth == 0 { 1 } else { 0 };
            if self.snake.iter().take(checked).any(|c| *c == new_head) {
                self.death_reason = "You bit yourself";
                self.game_over();
                return;
            }
        }

        self.snake.push_front(new_head);
        if self.pending_growth > 0 {
            self.pending_growth -= 1;
        } else {
            self.snake.pop_back();
        }

        // virus: consumed on contact, costs a life (or the shield)
        if let Some(i) = self.viruses.iter().position(|v| v.position == new_head) {
            self.viruses.remove(i);
            self.hurt(new_head);
            if self.state != GameState::Playing {
                return;
            }
        }

        // enemy snake
        if self.ghost <= 0.0 {
            let hit = self.enemies.iter().enumerate().find_map(|(i, e)| {
                if !e.alive {
                    return None;
                }
                return e
                    .body
                    .iter()
                    .position(|c| *c == new_head)
                    .map(|index| (i, index));
            });
            if let Some((enemy, index)) = hit {
                // boosting into an enemy's body rams it in half; its head (or a slow hit) hurts
                if self.boosting && index > 0 {
                    self.ram(enemy, index, new_head);
                } else {
                    self.hurt(new_head);
                    if self.state != GameState::Playing {
                        return;
                    }
                }
            }
        }

        // egg
        if let Some(i) = self.eggs.iter().position(|e| e.position == new_head) {
            let credit = match self.eggs[i].class {
                CollectibleType::Egg { special: true } => 3,
                _ => 1,
            };
            self.sfx(if credit == 3 { Sfx::EatGolden } else { Sfx::Eat });
            self.pending_growth += credit as usize;
            self.boost_meter = (self.boost_meter + BOOST_EGG_BONUS).min(1.0);
            self.respawn_egg(i);
            self.add_score(credit, new_head);
        }

        // power-up
        if let Some(i) = self.powers.iter().position(|p| p.position == new_head) {
            let power = self.powers.remove(i);
            if let CollectibleType::Power(kind) = power.class {
                match kind {
                    PowerKind::Shield => self.shield = true,
                    PowerKind::Ghost => self.ghost = GHOST_TIME,
                    PowerKind::Freeze => self.freeze = FREEZE_TIME,
                    PowerKind::Double => self.double = DOUBLE_TIME,
                }
                self.sfx(Sfx::Power);
                self.popup(new_head, kind.name(), kind.color());
            }
        }
    }

    /// The player took a hit from a virus or an enemy.
    fn hurt(&mut self, at: Point) {
        if self.damage_flash > 0.0 {
            return;
        }
        self.damage_flash = DAMAGE_FLASH_TIME;

        if self.shield {
            self.shield = false;
            self.sfx(Sfx::ShieldBreak);
            self.popup(at, "SHIELD BROKE", PowerKind::Shield.color());
            return;
        }

        self.lives -= 1;
        self.sfx(Sfx::Hurt);
        self.popup(at, "-1 LIFE", Color::RGB(255, 70, 90));
        if self.lives == 0 {
            self.death_reason = "Out of lives";
            self.game_over();
            return;
        }
        let keep = self.snake.len().saturating_sub(HIT_TAIL_LOSS).max(2);
        self.snake.truncate(keep);
        self.pending_growth = 0;
    }

    /// Cuts an enemy snake in half at `index`, scoring the severed cells.
    fn ram(&mut self, enemy: usize, index: usize, at: Point) {
        let removed = (self.enemies[enemy].body.len() - index) as u32;
        self.enemies[enemy].body.truncate(index);
        self.sfx(Sfx::Ram);
        self.popup(at, "RAM!", Color::RGB(255, 140, 0));
        self.add_score(removed, at);
        if self.enemies[enemy].body.len() < 3 {
            self.kill_enemy(enemy);
        }
    }

    fn kill_enemy(&mut self, index: usize) {
        self.enemies[index].alive = false;
        let at = self.enemies[index]
            .body
            .front()
            .copied()
            .unwrap_or(self.snake[0]);
        self.add_score(ENEMY_CRASH_BONUS, at);
        // it leaves a golden egg behind
        if self.eggs.len() < 6 && !self.occupied(at) {
            let mut egg = Collectible::new(at, CollectibleType::Egg { special: true });
            egg.age = 0.0;
            self.eggs.push(egg);
        }
    }

    fn add_score(&mut self, amount: u32, at: Point) {
        let gained = if self.double > 0.0 {
            amount * 2
        } else {
            amount
        };
        self.score += gained;
        self.popup(at, &format!("+{}", gained), Color::YELLOW);

        let level = self.score / SCORE_PER_LEVEL + 1;
        if level > self.level {
            self.level = level;
            self.sfx(Sfx::LevelUp);
            self.spawn_viruses();
            self.popup(offset(at, 0, -2), &format!("LEVEL {}", level), Color::WHITE);
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

    fn game_over(&mut self) {
        self.state = GameState::GameOver;
        self.sfx(Sfx::GameOver);
        self.over_age = 0.0;
        self.boosting = false;
        if self.score > self.high_score {
            self.high_score = self.score;
            self.new_high_score = true;
            save_high_score(self.high_score);
        }
    }

    // ---- enemy snakes ----

    fn try_spawn_enemy(&mut self) {
        let head = self.snake[0];
        for _ in 0..80 {
            let dir = Direction::ALL[rand::random_range(0..4)];
            let start = Point::new(rand::random_range(0..COLS), rand::random_range(0..ROWS));
            if wrapped_dist(start, head) < 20 {
                continue;
            }
            let (dx, dy) = dir.delta();
            let body: VecDeque<Point> = (0..ENEMY_START_LEN as i32)
                .map(|i| offset(start, -dx * i, -dy * i))
                .collect();
            if body.iter().all(|c| !self.occupied(*c)) {
                let n = self.enemies_spawned;
                self.enemies_spawned += 1;
                self.enemies.push(Enemy::new(body, dir, n % 3, n % 2 == 1));
                return;
            }
        }
    }

    fn step_enemies(&mut self, delta: f64) {
        if self.freeze > 0.0 {
            return;
        }
        let interval = self.enemy_interval();
        for i in 0..self.enemies.len() {
            self.enemies[i].timer += delta;
            while self.enemies[i].alive && self.enemies[i].timer >= interval {
                self.enemies[i].timer -= interval;
                self.step_enemy(i);
            }
        }
        self.enemies.retain(|e| e.alive);
    }

    fn step_enemy(&mut self, i: usize) {
        let Some(dir) = self.enemy_choose_dir(i) else {
            // boxed in: it crashes
            self.sfx(Sfx::Crash);
            self.popup(self.enemies[i].head(), "CRASH!", Color::RGB(255, 140, 0));
            self.kill_enemy(i);
            return;
        };

        let new_head = step(self.enemies[i].head(), dir);
        let enemy = &mut self.enemies[i];
        enemy.dir = dir;
        enemy.body.push_front(new_head);
        if enemy.pending_growth > 0 {
            enemy.pending_growth -= 1;
        } else {
            enemy.body.pop_back();
        }

        if let Some(egg) = self.eggs.iter().position(|e| e.position == new_head) {
            let bonus = match self.eggs[egg].class {
                CollectibleType::Egg { special: true } => 3,
                _ => 1,
            };
            if self.enemies[i].body.len() < ENEMY_MAX_LEN {
                self.enemies[i].pending_growth += bonus;
            }
            self.respawn_egg(egg);
        }
    }

    fn enemy_blocked(&self, p: Point) -> bool {
        return self.snake.contains(&p)
            || self.enemies.iter().any(|e| e.body.contains(&p))
            || self.viruses.iter().any(|v| v.position == p);
    }

    fn enemy_choose_dir(&self, i: usize) -> Option<Direction> {
        let enemy = &self.enemies[i];
        let head = enemy.head();
        let player = self.snake[0];

        let target = if enemy.hunter && wrapped_dist(head, player) < 30 {
            // aim ahead of the player to cut them off
            let (dx, dy) = self.dir.delta();
            offset(player, dx * 4, dy * 4)
        } else {
            self.eggs
                .iter()
                .map(|e| e.position)
                .min_by_key(|p| wrapped_dist(head, *p))
                .unwrap_or(player)
        };

        let mut best: Option<(i32, Direction)> = None;
        for dir in Direction::ALL {
            if dir == enemy.dir.opposite() {
                continue;
            }
            let next = step(head, dir);
            if self.enemy_blocked(next) {
                continue;
            }
            // avoid walking into dead ends, but not perfectly, so they can still be trapped
            let exits = Direction::ALL
                .iter()
                .filter(|d| !self.enemy_blocked(step(next, **d)))
                .count();
            let mut score = wrapped_dist(next, target) * 10 + rand::random_range(0..12);
            if exits == 0 {
                score += 200;
            }
            if best.map_or(true, |(s, _)| score < s) {
                best = Some((score, dir));
            }
        }
        return best.map(|(_, d)| d);
    }

    // ---- spawning ----

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

    fn occupied(&self, p: Point) -> bool {
        return self.snake.contains(&p)
            || self.enemies.iter().any(|e| e.body.contains(&p))
            || self.eggs.iter().any(|e| e.position == p)
            || self.viruses.iter().any(|v| v.position == p)
            || self.powers.iter().any(|v| v.position == p);
    }

    /// A random cell not occupied by anything, at least `head_margin` cells from the head.
    fn free_cell(&self, head_margin: i32) -> Point {
        let head = self.snake[0];
        loop {
            let p = Point::new(rand::random_range(0..COLS), rand::random_range(0..ROWS));
            if wrapped_dist(p, head) > head_margin && !self.occupied(p) {
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
    // best effort: failing to save shouldn't crash the game (and tests mustn't touch the file)
    if cfg!(not(test)) {
        let _ = fs::write(HIGH_SCORE_FILE, score.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays thousands of frames with random input at every level, looking for panics.
    #[test]
    fn random_play_never_panics() {
        let mut game = Game::new();
        for frame in 0..60_000u32 {
            if game.state != GameState::Playing {
                game.update(1.0);
                game.start();
            }
            if frame % 9 == 0 {
                let dir = Direction::ALL[rand::random_range(0..4)];
                game.turn(dir);
            }
            game.set_boost(rand::random_bool(0.5));
            if frame % 3000 == 0 {
                game.score += 12; // push through levels quickly
                game.add_score(1, Point::new(0, 0));
            }
            game.update(0.016);
            game.sounds.clear();
            assert!(game.snake.len() >= 2);
            assert!(game.enemies.iter().all(|e| !e.body.is_empty()));
        }
    }
}
