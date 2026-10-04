use std::collections::VecDeque;
use std::fs;
use std::io::{Read, Write};

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use sdl2::rect::Point;
use serde::{Deserialize, Serialize};

use super::*;
use crate::constants::GAMEDUMP_FILE;

/// name of the single entry inside the zip archive
const DUMP_ENTRY: &str = "gamedump.json";

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

/// Everything persisted between runs: the high score always, plus an unfinished round.
#[derive(Serialize, Deserialize, Default)]
pub struct GameDump {
    pub high_score: u32,
    #[serde(default)]
    pub save: Option<SaveData>,
}

fn read_u16(b: &[u8], at: usize) -> Option<u16> {
    return Some(u16::from_le_bytes(b.get(at..at + 2)?.try_into().ok()?));
}

fn read_u32(b: &[u8], at: usize) -> Option<u32> {
    return Some(u32::from_le_bytes(b.get(at..at + 4)?.try_into().ok()?));
}

fn deflate(data: &[u8]) -> Vec<u8> {
    let mut enc = DeflateEncoder::new(Vec::new(), Compression::default());
    enc.write_all(data).expect("in-memory deflate");
    return enc.finish().expect("in-memory deflate");
}

fn inflate(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    DeflateDecoder::new(data).read_to_end(&mut out).ok()?;
    return Some(out);
}

/// Wraps data in a minimal zip archive under a single deflated entry.
fn encode_zip(name: &str, data: &[u8]) -> Vec<u8> {
    let compressed = deflate(data);
    let crc = crc32fast::hash(data);
    let name = name.as_bytes();
    let mut out = Vec::new();

    // local file header
    out.extend_from_slice(&0x0403_4b50u32.to_le_bytes());
    out.extend_from_slice(&20u16.to_le_bytes()); // version needed
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&8u16.to_le_bytes()); // deflate
    out.extend_from_slice(&0u16.to_le_bytes()); // mod time
    out.extend_from_slice(&0u16.to_le_bytes()); // mod date
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&(name.len() as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // extra len
    out.extend_from_slice(name);
    out.extend_from_slice(&compressed);

    // central directory
    let cd_offset = out.len() as u32;
    out.extend_from_slice(&0x0201_4b50u32.to_le_bytes());
    out.extend_from_slice(&20u16.to_le_bytes()); // version made by
    out.extend_from_slice(&20u16.to_le_bytes()); // version needed
    out.extend_from_slice(&0u16.to_le_bytes()); // flags
    out.extend_from_slice(&8u16.to_le_bytes()); // deflate
    out.extend_from_slice(&0u16.to_le_bytes()); // mod time
    out.extend_from_slice(&0u16.to_le_bytes()); // mod date
    out.extend_from_slice(&crc.to_le_bytes());
    out.extend_from_slice(&(compressed.len() as u32).to_le_bytes());
    out.extend_from_slice(&(data.len() as u32).to_le_bytes());
    out.extend_from_slice(&(name.len() as u16).to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // extra len
    out.extend_from_slice(&0u16.to_le_bytes()); // comment len
    out.extend_from_slice(&0u16.to_le_bytes()); // disk start
    out.extend_from_slice(&0u16.to_le_bytes()); // internal attrs
    out.extend_from_slice(&0u32.to_le_bytes()); // external attrs
    out.extend_from_slice(&0u32.to_le_bytes()); // local header offset
    out.extend_from_slice(name);
    let cd_size = out.len() as u32 - cd_offset;

    // end of central directory
    out.extend_from_slice(&0x0605_4b50u32.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // disk
    out.extend_from_slice(&0u16.to_le_bytes()); // cd disk
    out.extend_from_slice(&1u16.to_le_bytes()); // entries on this disk
    out.extend_from_slice(&1u16.to_le_bytes()); // total entries
    out.extend_from_slice(&cd_size.to_le_bytes());
    out.extend_from_slice(&cd_offset.to_le_bytes());
    out.extend_from_slice(&0u16.to_le_bytes()); // comment len
    return out;
}

/// Extracts one named entry, verifying its size and CRC. Returns `None` on
/// anything that is not a well-formed archive.
fn decode_zip(bytes: &[u8], name: &str) -> Option<Vec<u8>> {
    let eocd = (0..=bytes.len().saturating_sub(22))
        .rev()
        .find(|&i| read_u32(bytes, i) == Some(0x0605_4b50))?;
    let count = read_u16(bytes, eocd + 10)?;
    let cd_off = read_u32(bytes, eocd + 16)? as usize;

    let mut p = cd_off;
    for _ in 0..count {
        if read_u32(bytes, p)? != 0x0201_4b50 {
            return None;
        }
        let method = read_u16(bytes, p + 10)?;
        let crc = read_u32(bytes, p + 16)?;
        let csize = read_u32(bytes, p + 20)? as usize;
        let usize_ = read_u32(bytes, p + 24)? as usize;
        let namelen = read_u16(bytes, p + 28)? as usize;
        let extralen = read_u16(bytes, p + 30)? as usize;
        let commentlen = read_u16(bytes, p + 32)? as usize;
        let local = read_u32(bytes, p + 42)? as usize;
        let entry = bytes.get(p + 46..p + 46 + namelen)?;
        if entry == name.as_bytes() {
            if read_u32(bytes, local)? != 0x0403_4b50 {
                return None;
            }
            let lnamelen = read_u16(bytes, local + 26)? as usize;
            let lextralen = read_u16(bytes, local + 28)? as usize;
            let data_off = local + 30 + lnamelen + lextralen;
            let comp = bytes.get(data_off..data_off + csize)?;
            let raw = match method {
                0 => comp.to_vec(),
                8 => inflate(comp)?,
                _ => return None,
            };
            if raw.len() != usize_ || crc32fast::hash(&raw) != crc {
                return None;
            }
            return Some(raw);
        }
        p += 46 + namelen + extralen + commentlen;
    }
    return None;
}

fn read_dump_at(path: &str) -> Option<GameDump> {
    let bytes = fs::read(path).ok()?;
    let json = decode_zip(&bytes, DUMP_ENTRY)?;
    return serde_json::from_slice(&json).ok();
}

fn write_dump_at(path: &str, dump: &GameDump) {
    if let Ok(json) = serde_json::to_vec(dump) {
        let _ = fs::write(path, encode_zip(DUMP_ENTRY, &json));
    }
}

fn read_dump() -> Option<GameDump> {
    return read_dump_at(GAMEDUMP_FILE);
}

/// failing to save shouldn't crash the game, and tests never touch the file.
fn write_dump(dump: &GameDump) {
    if cfg!(test) {
        return;
    }
    write_dump_at(GAMEDUMP_FILE, dump);
}

/// High score from the dump, or 0 if there is none / it can't be read.
pub fn load_high_score() -> u32 {
    if cfg!(test) {
        return 0;
    }
    return read_dump().map(|d| d.high_score).unwrap_or(0);
}

/// Records a finished round: the high score is kept, the resumable round is dropped.
pub fn finish_round(high_score: u32) {
    if cfg!(test) {
        return;
    }
    let mut dump = read_dump().unwrap_or_default();
    dump.high_score = dump.high_score.max(high_score);
    dump.save = None;
    write_dump(&dump);
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
    let data = read_dump()?.save?;
    return if data.is_valid_for(grid) {
        Some(data)
    } else {
        None
    };
}

/// Drops the resumable round but keeps the high score.
pub fn delete_save() {
    if cfg!(test) {
        return;
    }
    let mut dump = read_dump().unwrap_or_default();
    dump.save = None;
    write_dump(&dump);
}

impl Game {
    /// (score, level) of the saved round, if there is one to continue.
    pub fn save_summary(&self) -> Option<(u32, u32)> {
        return self.saved.as_ref().map(|s| (s.score, s.level));
    }

    /// Writes the current round and the high score to the dump. Only a round in
    /// progress is worth keeping.
    pub fn save_to_disk(&self) {
        if !matches!(self.state, GameState::Playing | GameState::Paused) {
            return;
        }

        write_dump(&GameDump {
            high_score: self.high_score,
            save: Some(self.snapshot()),
        });
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

    #[test]
    fn dump_zip_round_trips() {
        let dump = GameDump {
            high_score: 42,
            save: None,
        };
        let json = serde_json::to_vec(&dump).unwrap();
        let zip = encode_zip(DUMP_ENTRY, &json);
        // a real zip starts with the local file header signature
        assert_eq!(&zip[..4], &0x0403_4b50u32.to_le_bytes());
        assert_eq!(decode_zip(&zip, DUMP_ENTRY).unwrap(), json);
        // wrong entry name and truncated input are rejected
        assert!(decode_zip(&zip, "other.json").is_none());
        assert!(decode_zip(&zip[..zip.len() - 1], DUMP_ENTRY).is_none());
    }

    #[test]
    fn dump_file_round_trips_high_score_and_round() {
        let grid = Grid::for_screen(WINDOW_W, WINDOW_H);
        let mut game = Game::new(grid);
        game.start();
        for _ in 0..300 {
            game.update(0.016);
        }
        game.sounds.clear();
        assert!(matches!(game.state, GameState::Playing));

        let path = std::env::temp_dir().join(format!("snake_dump_{}.gamedump", std::process::id()));
        let path = path.to_str().unwrap();
        write_dump_at(
            path,
            &GameDump {
                high_score: 1234,
                save: Some(game.snapshot()),
            },
        );

        let raw = fs::read(path).unwrap();
        assert_eq!(
            &raw[..4],
            &0x0403_4b50u32.to_le_bytes(),
            "on-disk file is a zip"
        );

        let dump = read_dump_at(path).unwrap();
        assert_eq!(dump.high_score, 1234);
        let save = dump.save.unwrap();
        assert!(save.is_valid_for(grid));
        assert_eq!(save.score, game.score);
        assert_eq!(save.snake.len(), game.snake.len());

        // a corrupt file reads back as nothing, not a panic
        fs::write(path, b"not a zip").unwrap();
        assert!(read_dump_at(path).is_none());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn dump_zip_preserves_high_score_and_save_flag() {
        let dump = GameDump {
            high_score: 7,
            save: None,
        };
        let text = serde_json::to_string(&dump).unwrap();
        let data: GameDump = serde_json::from_str(&text).unwrap();
        assert_eq!(data.high_score, 7);
        assert!(data.save.is_none());
        // old dumps without the field still parse
        let data: GameDump = serde_json::from_str(r#"{"high_score":3}"#).unwrap();
        assert!(data.save.is_none());
    }
}
