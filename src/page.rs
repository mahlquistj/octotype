use crossterm::event::Event;
use ratatui::{Frame, layout::Rect, text::Line};

pub mod error;
pub mod loadscreen;
pub mod menu;
pub mod session;

pub use error::Error;
pub use loadscreen::Loading;
pub use menu::Menu;
pub use session::{Stats, TypingSession};

use crate::app::{Message, State};

macro_rules! make_page_enum {
    ($($t:tt),*) => {
        pub enum Page {
            $(
                $t(Box<$t>),
            )*
        }

        $(
            impl From<$t> for Page {
                fn from(value: $t) -> Page {
                    Page::$t(Box::new(value))
                }
            }
        )*
    };
}

make_page_enum!(Menu, Loading, Error, Stats, TypingSession);

impl Page {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, state: &State) {
        match self {
            Self::Menu(page) => page.render(frame, area, state),
            Self::Loading(page) => page.render(frame, area, &state.config),
            Self::TypingSession(page) => page.render(frame, area, &state.config),
            Self::Stats(page) => page.render(frame, area, &state.config),
            Self::Error(page) => page.render(frame, area, &state.config),
        }
    }

    pub fn render_top(&mut self, state: &State) -> Option<Line<'_>> {
        match self {
            Self::Menu(_) => None,
            Self::Loading(_) => None,
            Self::TypingSession(page) => page.render_top(&state.config),
            Self::Stats(page) => page.render_top(&state.config),
            Self::Error(page) => page.render_top(&state.config),
        }
    }

    pub fn handle_events(&mut self, event: &Event, state: &State) -> Option<Message> {
        match self {
            Self::Menu(page) => page.handle_events(event, state),
            Self::Loading(_) => None,
            Self::TypingSession(page) => page.handle_events(event, &state.config),
            Self::Stats(page) => page.handle_events(event, &state.config),
            Self::Error(page) => page.handle_events(event, &state.config),
        }
    }

    pub fn poll(&mut self, state: &State) -> Option<Message> {
        match self {
            Self::Menu(_) => None,
            Self::Loading(page) => page.poll(&state.config),
            Self::TypingSession(page) => page.poll(&state.config),
            Self::Stats(_) => None,
            Self::Error(_) => None,
        }
    }
}
