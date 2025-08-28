use crossterm::event::Event;
use derive_more::From;
use ratatui::{Frame, layout::Rect, text::Line};

pub mod error;
pub mod loadscreen;
pub mod menu;
pub mod session;

pub use error::Error;
pub use loadscreen::Loading;
pub use menu::Menu;
pub use session::{Stats, TypingSession};

use crate::{app::Message, config::Config};

#[derive(From)]
pub enum Page {
    Menu(Menu),
    LoadingScreen(Loading),
    TypingSession(TypingSession),
    Stats(Stats),
    Error(Error),
}

impl Page {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        match self {
            Self::Menu(page) => page.render(frame, area, config),
            Self::LoadingScreen(page) => page.render(frame, area, config),
            Self::TypingSession(page) => page.render(frame, area, config),
            Self::Stats(page) => page.render(frame, area, config),
            Self::Error(page) => page.render(frame, area, config),
        }
    }

    pub fn render_top(&mut self, config: &Config) -> Option<Line> {
        match self {
            Self::Menu(_) => None,
            Self::LoadingScreen(_) => None,
            Self::TypingSession(page) => page.render_top(config),
            Self::Stats(page) => page.render_top(config),
            Self::Error(page) => page.render_top(config),
        }
    }

    pub fn handle_events(&mut self, event: &Event, config: &Config) -> Option<Message> {
        match self {
            Self::Menu(page) => page.handle_events(event, config),
            Self::LoadingScreen(_) => None,
            Self::TypingSession(page) => page.handle_events(event, config),
            Self::Stats(page) => page.handle_events(event, config),
            Self::Error(page) => page.handle_events(event, config),
        }
    }

    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        match self {
            Self::Menu(_) => None,
            Self::LoadingScreen(page) => page.poll(config),
            Self::TypingSession(page) => page.poll(config),
            Self::Stats(_) => None,
            Self::Error(_) => None,
        }
    }
}
