use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
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
