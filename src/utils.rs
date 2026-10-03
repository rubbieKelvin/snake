use sdl2::{pixels::Color, rect::Point, render::WindowCanvas, ttf::Font};

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

/// Renders text horizontally centered on `center_x`.
pub fn render_text_centered(
    text: &str,
    center_x: i32,
    y: i32,
    font: &Font,
    canvas: &mut WindowCanvas,
    color: Color,
) -> Result<(), String> {
    let (w, _) = font.size_of(text).map_err(|e| e.to_string())?;
    return render_text(
        text,
        Point::new(center_x - w as i32 / 2, y),
        font,
        canvas,
        color,
    );
}
