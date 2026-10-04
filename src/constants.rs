pub const CELL: u32 = 30;

// design size
pub const WINDOW_W: u32 = 1400;
pub const WINDOW_H: u32 = 800;

pub const HUD_H: i32 = 90;

pub const START_LIVES: u8 = 3;
pub const SCORE_PER_LEVEL: u32 = 10;

pub const BASE_STEP_INTERVAL: f64 = 0.12;
pub const MIN_STEP_INTERVAL: f64 = 0.07;
pub const STEP_SPEEDUP_PER_LEVEL: f64 = 0.024;

pub const SPECIAL_EGG_LIFETIME: f64 = 6.0;
pub const HIT_TAIL_LOSS: usize = 3;
pub const DAMAGE_FLASH_TIME: f64 = 1.0;
pub const RESTART_LOCKOUT: f64 = 0.8;
pub const POPUP_LIFETIME: f64 = 1.1;

// boost
pub const BOOST_SPEED_FACTOR: f64 = 0.5;
pub const BOOST_DRAIN: f64 = 0.45;
pub const BOOST_REGEN: f64 = 0.18;
pub const BOOST_RESUME: f64 = 0.25;
pub const BOOST_EGG_BONUS: f64 = 0.15;

// power-ups
pub const POWER_LIFETIME: f64 = 10.0;
pub const POWER_SPAWN_INTERVAL: f64 = 8.0;
pub const GHOST_TIME: f64 = 6.0;
pub const FREEZE_TIME: f64 = 6.0;
pub const DOUBLE_TIME: f64 = 10.0;

// enemy snakes
pub const MAX_ENEMIES: usize = 3;
pub const ENEMY_START_LEN: usize = 4;
pub const ENEMY_MAX_LEN: usize = 14;
pub const ENEMY_SPAWN_DELAY: f64 = 3.0;
pub const ENEMY_CRASH_BONUS: u32 = 5;
pub const ENEMY_HIT_FLASH: f64 = 0.25;

// shooting
pub const MAX_AMMO: u8 = 10;
pub const AMMO_PICKUP: u8 = 3;
pub const AMMO_SPAWN_INTERVAL: f64 = 9.0;
pub const MAX_AMMO_DROPS: usize = 2;
/// seconds per cell travelled
pub const BULLET_STEP_INTERVAL: f64 = 0.025;
/// cells a bullet flies before fizzling out
pub const BULLET_RANGE: i32 = 25;
pub const FIRE_COOLDOWN: f64 = 0.18;
pub const VIRUS_SHOT_BONUS: u32 = 1;

// game save file
pub const GAMEDUMP_FILE: &str = ".gamedump";
