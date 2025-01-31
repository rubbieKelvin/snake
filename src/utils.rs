use crate::constants::*;
use sdl2::{pixels::Color, rect::Point, rect::Rect, render::WindowCanvas, ttf::Font};

pub fn random_position_on_screen() -> Point {
    let x = rand::random_range(0..(WINDOW_W / SNAKE_W) as i32);
    let y = rand::random_range(0..(WINDOW_H / SNAKE_H) as i32);
    return Point::new(x * SNAKE_W as i32, y * SNAKE_H as i32);
}

pub fn rect_from_point(point: &Point, w: u32, h: u32) -> Rect {
    return Rect::new(point.x, point.y, w, h);
}

pub fn render_text(
    text: &str,
    position: Point,
    font: &Font,
    canvas: &mut WindowCanvas,
    color: Color,
) -> Result<(), String> {
    let surf = font
        .render(text)
        .blended(color)
        .map_err(|e| e.to_string())?;

    let texture_creator = canvas.texture_creator();
    let texture = surf
        .as_texture(&texture_creator)
        .map_err(|e| e.to_string())?;

    let mut rect = surf.rect();
    rect.set_x(position.x);
    rect.set_y(position.y);

    canvas.copy(&texture, None, rect)?;
    return Ok(());
}
