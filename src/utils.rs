use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
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

/// A trait defining helper methods for keyevents
pub trait KeyEventHelper {
    /// Returns true if the keyevent contains a pressed key
    fn is_press(&self) -> bool;

    /// Returns true if the keyevent contains the given modifiers
    fn has_mods(&self, mods: KeyModifiers) -> bool;

    /// Returns true if the keyevent contains a character that matches the input
    fn is_char(&self, character: char) -> bool;

    /// Returns true if the keyevent matches the given character, and is being pressed
    fn is_press_char(&self, character: char) -> bool {
        self.is_press() && self.is_char(character)
    }

    /// Returns true if the keyevent matches the given character, and is being pressed with CTRL as
    /// a modifier.
    fn is_ctrl_press_char(&self, character: char) -> bool {
        self.has_mods(KeyModifiers::CONTROL) && self.is_press_char(character)
    }
}

impl KeyEventHelper for KeyEvent {
    fn is_press(&self) -> bool {
        self.kind == KeyEventKind::Press
    }

    fn is_char(&self, character: char) -> bool {
        self.code == KeyCode::Char(character)
    }

    fn has_mods(&self, mods: KeyModifiers) -> bool {
        self.modifiers.contains(mods)
    }
}
