use sdl2::{
    pixels::Color,
    rect::{Point, Rect},
    render::{BlendMode, WindowCanvas},
    ttf::Font,
};

use crate::constants::*;
use crate::game::Game;
use crate::objs::*;
use crate::utils::{render_text, render_text_centered};

pub struct Fonts<'a> {
    pub tiny: Font<'a, 'static>,
    pub small: Font<'a, 'static>,
    pub normal: Font<'a, 'static>,
    pub big: Font<'a, 'static>,
}

const ENEMY_PALETTE: [(u8, u8, u8); 3] = [(255, 140, 0), (130, 90, 255), (30, 110, 210)];
const HEART: [&str; 6] = [
    ".##.##.", "#######", "#######", ".#####.", "..###..", "...#...",
];

// small drawing helpers
fn cell_rect(grid: &Grid, p: Point) -> Rect {
    let o = grid.origin();
    return Rect::new(o.x + p.x * CELL as i32, o.y + p.y * CELL as i32, CELL, CELL);
}

fn inset(r: Rect, by: i32) -> Rect {
    return Rect::new(
        r.x + by,
        r.y + by,
        (r.w - 2 * by).max(0) as u32,
        (r.h - 2 * by).max(0) as u32,
    );
}

fn fill(canvas: &mut WindowCanvas, color: Color, rect: Rect) {
    canvas.set_draw_color(color);
    let _ = canvas.fill_rect(rect);
}

fn outline(canvas: &mut WindowCanvas, color: Color, rect: Rect) {
    canvas.set_draw_color(color);
    let _ = canvas.draw_rect(rect);
}

fn text(canvas: &mut WindowCanvas, font: &Font, s: &str, x: i32, y: i32, color: Color) {
    let _ = render_text(s, Point::new(x, y), font, canvas, color);
}

fn text_right(canvas: &mut WindowCanvas, font: &Font, s: &str, right: i32, y: i32, color: Color) {
    if let Ok((w, _)) = font.size_of(s) {
        text(canvas, font, s, right - w as i32, y, color);
    }
}

fn text_center(canvas: &mut WindowCanvas, font: &Font, s: &str, cx: i32, y: i32, color: Color) {
    let _ = render_text_centered(s, cx, y, font, canvas, color);
}

/// Centered text with a drop shadow, for titles.
fn title(canvas: &mut WindowCanvas, font: &Font, s: &str, cx: i32, y: i32, color: Color) {
    text_center(canvas, font, s, cx + 3, y + 4, Color::RGBA(0, 0, 0, 160));
    text_center(canvas, font, s, cx, y, color);
}

fn panel(canvas: &mut WindowCanvas, rect: Rect) {
    fill(canvas, Color::RGBA(10, 25, 15, 215), rect);
    outline(canvas, Color::RGBA(255, 255, 255, 70), rect);
}

fn bar(canvas: &mut WindowCanvas, rect: Rect, frac: f64, color: Color) {
    fill(canvas, Color::RGBA(0, 0, 0, 140), rect);
    let w = (rect.w as f64 * frac.clamp(0.0, 1.0)) as u32;
    if w > 0 {
        fill(canvas, color, Rect::new(rect.x, rect.y, w, rect.h as u32));
    }
    outline(canvas, Color::RGBA(255, 255, 255, 90), rect);
}

fn heart(canvas: &mut WindowCanvas, x: i32, y: i32, scale: i32, color: Color) {
    for (row, line) in HEART.iter().enumerate() {
        for (col, ch) in line.chars().enumerate() {
            if ch == '#' {
                let r = Rect::new(
                    x + col as i32 * scale,
                    y + row as i32 * scale,
                    scale as u32,
                    scale as u32,
                );
                fill(canvas, color, r);
            }
        }
    }
}

fn swatch(canvas: &mut WindowCanvas, fonts: &Fonts, x: i32, y: i32, color: Color, label: &str) {
    let r = Rect::new(x, y, 24, 24);
    fill(canvas, color, r);
    outline(canvas, Color::RGB(20, 20, 20), r);
    if !label.is_empty() {
        text_center(
            canvas,
            &fonts.tiny,
            label,
            x + 12,
            y + 3,
            Color::RGB(20, 20, 20),
        );
    }
}

// world
fn draw_world(canvas: &mut WindowCanvas, game: &Game, fonts: &Fonts) {
    let grid = game.grid;
    // checkerboard: fill once, then paint only the alternate cells
    canvas.set_draw_color(Color::RGB(38, 158, 70));
    canvas.clear();
    canvas.set_draw_color(Color::RGB(45, 172, 80));
    for y in 0..grid.rows {
        for x in 0..grid.cols {
            if (x + y) % 2 == 0 {
                let _ = canvas.fill_rect(cell_rect(&grid, Point::new(x, y)));
            }
        }
    }

    let pulse = ((game.clock * 6.0).sin() * 0.5 + 0.5) as f32;

    for egg in &game.eggs {
        if let CollectibleType::Egg { special } = egg.class {
            let rect = cell_rect(&grid, egg.position);
            // special eggs blink while about to expire
            let urgent = special && egg.age > SPECIAL_EGG_LIFETIME - 2.0;
            if urgent && (egg.age * 6.0) as i32 % 2 == 0 {
                continue;
            }
            let visual = if special { rect } else { inset(rect, 5) };
            fill(
                canvas,
                if special { Color::YELLOW } else { Color::CYAN },
                visual,
            );
            outline(canvas, Color::RGB(20, 20, 20), visual);
        }
    }

    for power in &game.powers {
        if let CollectibleType::Power(kind) = power.class {
            if power.age > POWER_LIFETIME - 3.0 && (power.age * 8.0) as i32 % 2 == 0 {
                continue;
            }
            let rect = cell_rect(&grid, power.position);
            let glow = (60.0 + 120.0 * pulse) as u8;
            fill(canvas, Color::RGBA(255, 255, 255, glow), inset(rect, -3));
            swatch(
                canvas,
                fonts,
                rect.x - 2,
                rect.y - 2,
                kind.color(),
                kind.letter(),
            );
        }
    }

    for heart_item in &game.hearts {
        let rect = cell_rect(&grid, heart_item.position);
        // gentle bob so it reads as something to grab
        let bob = ((game.clock * 5.0).sin() * 1.5) as i32;
        let (x, y) = (rect.x + 3, rect.y + 4 + bob);
        heart(canvas, x + 1, y + 1, 2, Color::RGBA(0, 0, 0, 110));
        heart(canvas, x, y, 2, Color::RGB(255, 60, 95));
    }

    for virus in &game.viruses {
        let rect = cell_rect(&grid, virus.position);
        fill(canvas, Color::RGB(150, 0, 170), inset(rect, 3));
        fill(canvas, Color::RGB(230, 60, 90), inset(rect, 7));
        outline(canvas, Color::RGB(20, 20, 20), inset(rect, 3));
        // spikes
        let c = rect.center();
        for (dx, dy) in [(-9, 0), (8, 0), (0, -9), (0, 8)] {
            fill(
                canvas,
                Color::RGB(150, 0, 170),
                Rect::new(c.x + dx - 1, c.y + dy - 1, 3, 3),
            );
        }
    }

    draw_enemies(canvas, game);
    draw_player(canvas, game);

    // floating popups
    for popup in &game.popups {
        let x = grid.origin().x + popup.position.x * CELL as i32 + 10;
        let y = grid.origin().y + popup.position.y * CELL as i32 - (popup.age * 45.0) as i32 - 12;
        if let Ok((w, _)) = fonts.small.size_of(&popup.text) {
            let x = (x - w as i32 / 2).clamp(4, grid.width as i32 - w as i32 - 4);
            text(
                canvas,
                &fonts.small,
                &popup.text,
                x + 2,
                y + 2,
                Color::RGBA(0, 0, 0, 180),
            );
            text(canvas, &fonts.small, &popup.text, x, y, popup.color);
        }
    }
}

fn eyes(canvas: &mut WindowCanvas, grid: Grid, cell: Point, dir: Direction, color: Color) {
    let r = cell_rect(&grid, cell);
    let (dx, dy) = dir.delta();
    // two eyes offset perpendicular to the heading, pushed forward a bit
    let (px, py) = (dy.abs(), dx.abs());
    for side in [-1, 1] {
        let ex = r.x + 8 + dx * 4 + px * side * 5;
        let ey = r.y + 8 + dy * 4 + py * side * 5;
        fill(canvas, color, Rect::new(ex, ey, 4, 4));
    }
}

fn draw_enemies(canvas: &mut WindowCanvas, game: &Game) {
    let grid = game.grid;
    let frozen = game.freeze > 0.0;
    for enemy in &game.enemies {
        let (r, g, b) = ENEMY_PALETTE[enemy.color_idx % ENEMY_PALETTE.len()];
        let len = enemy.body.len();
        for (i, cell) in enemy.body.iter().enumerate() {
            // fade toward the tail
            let shade = 1.0 - 0.45 * (i as f32 / len as f32);
            let color = if frozen {
                Color::RGB(
                    (170.0 * shade) as u8,
                    (225.0 * shade) as u8,
                    (255.0 * shade) as u8,
                )
            } else {
                Color::RGB(
                    (r as f32 * shade) as u8,
                    (g as f32 * shade) as u8,
                    (b as f32 * shade) as u8,
                )
            };
            let rect = cell_rect(&grid, *cell);
            fill(canvas, color, if i == 0 { rect } else { inset(rect, 1) });
            if i == 0 {
                outline(canvas, Color::RGB(20, 20, 20), rect);
            }
        }
        eyes(canvas, grid, enemy.head(), enemy.dir, Color::RGB(220, 20, 20));
    }
}

fn draw_player(canvas: &mut WindowCanvas, game: &Game) {
    let grid = game.grid;
    let len = game.snake.len();
    let hurt_flash = game.damage_flash > 0.0 && (game.damage_flash * 10.0) as i32 % 2 == 0;
    let dead = game.state == GameState::GameOver;
    // ghost makes the snake translucent
    let alpha = if game.ghost > 0.0 { 130 } else { 255 };

    for (index, cell) in game.snake.iter().enumerate() {
        let rect = cell_rect(&grid, *cell);
        let ratio = (index as f32 + 1.0) / len as f32;
        let iratio = ((len - index) as f32 + 1.0) / len as f32;
        let (mut r, mut g, mut b) = (
            (255.0 * ratio) as u8,
            100u8,
            (160.0 * iratio).min(255.0) as u8,
        );
        if hurt_flash {
            (r, g, b) = (255, 255, 255);
        } else if dead {
            (r, g, b) = (110, 110, 110);
        } else if game.boosting {
            // brighter while boosting
            r = r.saturating_add(40);
            g = g.saturating_add(90);
        }
        fill(canvas, Color::RGBA(r, g, b, alpha), rect);

        if game.shield {
            outline(canvas, PowerKind::Shield.color(), rect);
            outline(canvas, PowerKind::Shield.color(), inset(rect, 1));
        }
    }

    // boost glow around the head
    if game.boosting {
        if let Some(head) = game.snake.front() {
            outline(canvas, Color::YELLOW, inset(cell_rect(&grid, *head), -2));
        }
    }

    if let Some(head) = game.snake.front() {
        eyes(canvas, grid, *head, game.dir, Color::WHITE);
    }
}

// HUD: an opaque header above the board, never overlapping the playfield
fn draw_hud(canvas: &mut WindowCanvas, game: &Game, fonts: &Fonts) {
    let grid = game.grid;
    let w = grid.width as i32;
    let light = Color::RGB(235, 255, 240);
    let dim = Color::RGB(150, 190, 160);

    fill(
        canvas,
        Color::RGB(8, 22, 12),
        Rect::new(0, 0, grid.width, HUD_H as u32),
    );
    fill(
        canvas,
        Color::RGB(70, 110, 80),
        Rect::new(0, HUD_H - 2, grid.width, 2),
    );

    // score
    text(canvas, &fonts.tiny, "SCORE", 24, 10, dim);
    text(
        canvas,
        &fonts.big,
        &game.score.to_string(),
        24,
        18,
        light,
    );

    // boost meter
    let bx = 250;
    let label = if game.boost_exhausted {
        "RECHARGING"
    } else {
        "BOOST  [SPACE]"
    };
    text(
        canvas,
        &fonts.tiny,
        label,
        bx,
        14,
        if game.boost_exhausted {
            Color::RGB(255, 110, 110)
        } else {
            dim
        },
    );
    let bar_color = if game.boost_exhausted {
        Color::RGB(200, 70, 70)
    } else if game.boosting {
        Color::RGB(255, 230, 60)
    } else {
        Color::RGB(255, 170, 40)
    };
    bar(
        canvas,
        Rect::new(bx, 36, 220, 16),
        game.boost_meter,
        bar_color,
    );

    // level progress, centered
    let into_level = (game.score % SCORE_PER_LEVEL) as f64 / SCORE_PER_LEVEL as f64;
    text_center(
        canvas,
        &fonts.normal,
        &format!("LEVEL {}", game.level),
        w / 2 - 40,
        8,
        light,
    );
    bar(
        canvas,
        Rect::new(w / 2 - 190, 46, 300, 14),
        into_level,
        Color::RGB(90, 220, 255),
    );

    // active power-ups
    let mut active: Vec<(PowerKind, f64)> = Vec::new();
    if game.shield {
        active.push((PowerKind::Shield, 1.0));
    }
    if game.ghost > 0.0 {
        active.push((PowerKind::Ghost, game.ghost / GHOST_TIME));
    }
    if game.freeze > 0.0 {
        active.push((PowerKind::Freeze, game.freeze / FREEZE_TIME));
    }
    if game.double > 0.0 {
        active.push((PowerKind::Double, game.double / DOUBLE_TIME));
    }
    let slot_w = 100;
    let mut x = w / 2 + 150;
    for (kind, frac) in active {
        swatch(canvas, fonts, x, 14, kind.color(), kind.letter());
        bar(
            canvas,
            Rect::new(x, 46, slot_w as u32 - 10, 8),
            frac,
            kind.color(),
        );
        text(canvas, &fonts.tiny, kind.name(), x + 30, 16, dim);
        x += slot_w;
    }

    // lives + best
    for i in 0..START_LIVES as i32 {
        let color = if i < game.lives as i32 {
            Color::RGB(240, 50, 80)
        } else {
            Color::RGB(40, 55, 45)
        };
        heart(canvas, w - 24 - (i + 1) * 30 + 6, 14, 3, color);
    }
    text_right(
        canvas,
        &fonts.small,
        &format!("BEST {}", game.high_score),
        w - 24,
        46,
        dim,
    );
}

// overlays
fn draw_menu(canvas: &mut WindowCanvas, game: &Game, fonts: &Fonts) {
    let cx = WINDOW_W as i32 / 2;
    title(canvas, &fonts.big, "SNAKE", cx, 40, Color::WHITE);
    if (game.clock * 2.0) as i32 % 2 == 0 {
        text_center(
            canvas,
            &fonts.normal,
            "Press ENTER to start",
            cx,
            140,
            Color::YELLOW,
        );
    }

    let panel_rect = Rect::new(cx - 400, 205, 800, 410);
    panel(canvas, panel_rect);
    let x = panel_rect.x + 30;
    let mut y = panel_rect.y + 20;
    let white = Color::WHITE;

    let rows: Vec<(Color, &str, String)> = vec![
        (Color::CYAN, "", "Egg: +1 point, grow longer".into()),
        (
            Color::YELLOW,
            "",
            "Golden egg: +3, but it vanishes quickly".into(),
        ),
        (
            Color::RGB(230, 60, 90),
            "",
            "Virus: costs a life and part of your tail".into(),
        ),
        (
            Color::RGB(255, 60, 95),
            "",
            "Heart: restores a life (offered when a level starts)".into(),
        ),
        (
            Color::RGB(255, 140, 0),
            "",
            "Enemy snake: avoid it, or ram its body while boosting".into(),
        ),
    ];
    for (color, label, desc) in rows {
        swatch(canvas, fonts, x, y, color, label);
        text(canvas, &fonts.small, &desc, x + 40, y, white);
        y += 36;
    }
    y += 8;
    for kind in PowerKind::ALL {
        swatch(canvas, fonts, x, y, kind.color(), kind.letter());
        text(
            canvas,
            &fonts.small,
            &format!("{}: {}", kind.name(), kind.describe()),
            x + 40,
            y,
            white,
        );
        y += 36;
    }

    text_center(
        canvas,
        &fonts.small,
        "WASD / arrows: move     SPACE / SHIFT (hold): boost     P: pause     M: mute     ESC: quit",
        cx,
        panel_rect.bottom() + 20,
        Color::WHITE,
    );
    if game.high_score > 0 {
        text_center(
            canvas,
            &fonts.normal,
            &format!("Best: {}", game.high_score),
            cx,
            panel_rect.bottom() + 60,
            Color::YELLOW,
        );
    }
}

fn draw_paused(canvas: &mut WindowCanvas, fonts: &Fonts) {
    let cx = WINDOW_W as i32 / 2;
    panel(canvas, Rect::new(cx - 260, 270, 520, 220));
    title(canvas, &fonts.big, "PAUSED", cx, 285, Color::WHITE);
    text_center(
        canvas,
        &fonts.normal,
        "Press P or ENTER to resume",
        cx,
        400,
        Color::WHITE,
    );
}

fn draw_game_over(canvas: &mut WindowCanvas, game: &Game, fonts: &Fonts) {
    let cx = WINDOW_W as i32 / 2;
    panel(canvas, Rect::new(cx - 330, 190, 660, 400));
    title(
        canvas,
        &fonts.big,
        "GAME OVER",
        cx,
        205,
        Color::RGB(255, 90, 90),
    );
    text_center(
        canvas,
        &fonts.small,
        game.death_reason,
        cx,
        305,
        Color::RGB(200, 200, 200),
    );
    text_center(
        canvas,
        &fonts.normal,
        &format!("Score  {}", game.score),
        cx,
        350,
        Color::WHITE,
    );
    text_center(
        canvas,
        &fonts.normal,
        &format!("Level  {}", game.level),
        cx,
        390,
        Color::WHITE,
    );
    let best = if game.new_high_score {
        "NEW HIGH SCORE!".to_string()
    } else {
        format!("Best  {}", game.high_score)
    };
    text_center(canvas, &fonts.normal, &best, cx, 440, Color::YELLOW);
    if (game.clock * 2.0) as i32 % 2 == 0 {
        text_center(
            canvas,
            &fonts.normal,
            "Press ENTER to play again",
            cx,
            520,
            Color::WHITE,
        );
    }
}

pub fn render(canvas: &mut WindowCanvas, game: &Game, fonts: &Fonts) {
    canvas.set_blend_mode(BlendMode::Blend);
    draw_world(canvas, game, fonts);
    draw_hud(canvas, game, fonts);

    if game.state != GameState::Playing {
        fill(
            canvas,
            Color::RGBA(0, 0, 0, 140),
            Rect::new(0, 0, game.grid.width, game.grid.height),
        );
    }

    // the overlays are laid out for the design size, so center that area on the screen
    let x = (game.grid.width as i32 - WINDOW_W as i32) / 2;
    let y = (game.grid.height as i32 - WINDOW_H as i32) / 2;
    canvas.set_viewport(Rect::new(x, y, WINDOW_W, WINDOW_H));
    match game.state {
        GameState::Menu => draw_menu(canvas, game, fonts),
        GameState::Paused => draw_paused(canvas, fonts),
        GameState::GameOver => draw_game_over(canvas, game, fonts),
        GameState::Playing => {}
    }
    canvas.set_viewport(None);
}
