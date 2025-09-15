use ansi_colours::rgb_from_ansi256;
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    style::Color,
    widgets::{Block, BorderType},
};

/// Timestamp type for more clarity
pub type Timestamp = f64;

/// A block with a rounded border
pub const ROUNDED_BLOCK: Block = Block::bordered().border_type(BorderType::Rounded);

/// Creates a centered area within the given Rect respective to the horizontal and vertical
/// constriants.
pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area_horizontal] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical])
        .flex(Flex::Center)
        .areas(area_horizontal);
    area
}

/// Fades `color1` towards `color2` by the given percentage
pub fn fade(color1: Color, color2: Color, percentage: f32, is_foreground: bool) -> Color {
    let (r1, g1, b1) = color_to_rgb(color1, is_foreground);
    let (r2, g2, b2) = color_to_rgb(color2, is_foreground);

    let new_r = (r2 as f32 - r1 as f32)
        .mul_add(percentage, r1 as f32)
        .round() as u8;
    let new_g = (g2 as f32 - g1 as f32)
        .mul_add(percentage, g1 as f32)
        .round() as u8;
    let new_b = (b2 as f32 - b1 as f32)
        .mul_add(percentage, b1 as f32)
        .round() as u8;

    Color::Rgb(new_r, new_g, new_b)
}

pub fn color_to_rgb(color: Color, is_foreground: bool) -> (u8, u8, u8) {
    match color {
        Color::Rgb(r, g, b) => (r, g, b),
        Color::Indexed(idx) => rgb_from_ansi256(idx),
        Color::Black => rgb_from_ansi256(if is_foreground { 30 } else { 40 }),
        Color::Red => rgb_from_ansi256(if is_foreground { 31 } else { 41 }),
        Color::Green => rgb_from_ansi256(if is_foreground { 32 } else { 42 }),
        Color::Yellow => rgb_from_ansi256(if is_foreground { 33 } else { 43 }),
        Color::Blue => rgb_from_ansi256(if is_foreground { 34 } else { 44 }),
        Color::Magenta => rgb_from_ansi256(if is_foreground { 35 } else { 45 }),
        Color::Cyan => rgb_from_ansi256(if is_foreground { 36 } else { 46 }),
        Color::Gray => rgb_from_ansi256(if is_foreground { 37 } else { 47 }),
        Color::DarkGray => rgb_from_ansi256(if is_foreground { 90 } else { 100 }),
        Color::LightRed => rgb_from_ansi256(if is_foreground { 91 } else { 101 }),
        Color::LightGreen => rgb_from_ansi256(if is_foreground { 92 } else { 102 }),
        Color::LightYellow => rgb_from_ansi256(if is_foreground { 93 } else { 103 }),
        Color::LightBlue => rgb_from_ansi256(if is_foreground { 94 } else { 104 }),
        Color::LightMagenta => rgb_from_ansi256(if is_foreground { 95 } else { 105 }),
        Color::LightCyan => rgb_from_ansi256(if is_foreground { 96 } else { 106 }),
        Color::White => rgb_from_ansi256(if is_foreground { 97 } else { 107 }),
        Color::Reset => (0, 0, 0), // Default to black for Reset
    }
}
