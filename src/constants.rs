pub const CELL: u32 = 20;

pub const WINDOW_W: u32 = 1400;
pub const WINDOW_H: u32 = 800;

pub const COLS: i32 = (WINDOW_W / CELL) as i32;
pub const ROWS: i32 = (WINDOW_H / CELL) as i32;

pub const START_LIVES: u8 = 3;
pub const SCORE_PER_LEVEL: u32 = 10;

pub const BASE_STEP_INTERVAL: f64 = 0.18;
pub const MIN_STEP_INTERVAL: f64 = 0.07;
pub const STEP_SPEEDUP_PER_LEVEL: f64 = 0.012;

pub const SPECIAL_EGG_LIFETIME: f64 = 6.0;
pub const VIRUS_TAIL_LOSS: usize = 3;
pub const DAMAGE_FLASH_TIME: f64 = 1.0;

pub const HIGH_SCORE_FILE: &str = "highscore.txt";
