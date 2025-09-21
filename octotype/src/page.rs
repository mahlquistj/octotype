use crossterm::event::Event;
use ratatui::{Frame, layout::Rect, text::Line};

pub mod error;
pub mod loadscreen;
pub mod menu;
pub mod session;

pub use error::Error;
pub use loadscreen::Loading;
pub use menu::Menu;
pub use session::Session;

use crate::{app::Message, config::Config};

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

make_page_enum!(Menu, Loading, Error, Session);

impl Page {
    pub fn render(&mut self, frame: &mut Frame, area: Rect, config: &Config) {
        match self {
            Self::Menu(page) => page.render(frame, area, config),
            Self::Loading(page) => page.render(frame, area, config),
            Self::Session(page) => page.render(frame, area, config),
            // Self::Stats(page) => page.render(frame, area, config),
            Self::Error(page) => page.render(frame, area, config),
        }
    }

    pub fn render_top(&mut self, config: &Config) -> Option<Line<'_>> {
        match self {
            Self::Menu(_) => None,
            Self::Loading(_) => None,
            Self::Session(page) => page.render_top(config),
            // Self::Stats(page) => page.render_top(config),
            Self::Error(page) => page.render_top(config),
        }
    }

    pub fn handle_events(&mut self, event: &Event, config: &Config) -> Option<Message> {
        match self {
            Self::Menu(page) => page.handle_events(event, config),
            Self::Loading(_) => None,
            Self::Session(page) => page.handle_events(event, config),
            // Self::Stats(page) => page.handle_events(event, config),
            Self::Error(page) => page.handle_events(event, config),
        }
    }

    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        match self {
            Self::Menu(_) => None,
            Self::Loading(page) => page.poll(config),
            Self::Session(page) => page.poll(config),
            // Self::Stats(_) => None,
            Self::Error(_) => None,
        }
    }
}
