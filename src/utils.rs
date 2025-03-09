use std::thread::JoinHandle;

use crossterm::event::{Event, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Flex, Layout, Rect},
    widgets::{Block, BorderType},
    Frame,
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

pub enum Message {
    Quit,
    Show(Box<dyn Page + Send>),
    Await(JoinHandle<Result<Message, minreq::Error>>),
    ShowLoaded,
}

pub type EventResult = std::io::Result<Option<Message>>;

pub trait Page {
    fn render(&mut self, frame: &mut Frame, area: Rect);

    fn handle_events(&mut self, _event: &Event) -> EventResult {
        Ok(None)
    }

    fn boxed(self) -> Box<Self>
    where
        Self: Sized,
    {
        Box::new(self)
    }
}
