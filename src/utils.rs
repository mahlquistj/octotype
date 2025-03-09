use crossterm::event::{KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType},
};

pub type Timestamp = f64;

pub const ROUNDED_BLOCK: Block = Block::bordered().border_type(BorderType::Rounded);

pub fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area_horizontal] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical])
        .flex(Flex::Center)
        .areas(area_horizontal);
    area
}

/// A trait defining helper methods for keyevents
pub trait KeyEventHelper {
    fn is_press(&self) -> bool;
    fn has_ctrl_mod(&self) -> bool;

    fn is_ctrl_press(&self) -> bool {
        self.is_press() && self.has_ctrl_mod()
    }
}

impl KeyEventHelper for KeyEvent {
    fn is_press(&self) -> bool {
        self.kind == KeyEventKind::Press
    }

    fn has_ctrl_mod(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }
}
