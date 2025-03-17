use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    text::Line,
    widgets::{Block, BorderType},
    Frame,
};

use crate::config::Config;

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
    fn is_char(&self, character: char) -> bool;

    fn is_press_char(&self, character: char) -> bool {
        self.is_press() && self.is_char(character)
    }

    fn is_ctrl_press(&self) -> bool {
        self.is_press() && self.has_ctrl_mod()
    }

    fn is_ctrl_press_char(&self, character: char) -> bool {
        self.has_ctrl_mod() && self.is_press_char(character)
    }
}

impl KeyEventHelper for KeyEvent {
    fn is_press(&self) -> bool {
        self.kind == KeyEventKind::Press
    }

    fn is_char(&self, character: char) -> bool {
        self.code == KeyCode::Char(character)
    }

    fn has_ctrl_mod(&self) -> bool {
        self.modifiers.contains(KeyModifiers::CONTROL)
    }
}

pub enum Message {
    Quit,
    Show(Box<dyn Page + Send>),
}

pub trait Page {
    // Renders the page. Called every cycle
    fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config);

    // Renders a line in the top left of the window.
    // Called every cycle, before render
    fn render_top(&mut self, _config: &Config) -> Option<Line> {
        None
    }

    // Handles events for the page. Called every time an event appears
    fn handle_events(&mut self, _event: &Event, _config: &Config) -> Option<Message> {
        None
    }

    fn poll(&mut self, _config: &Config) -> Option<Message> {
        None
    }

    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
