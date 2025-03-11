use std::{
    fmt::Display,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use ratatui::{layout::Constraint, style::Style};
use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse, BRAILLE_SIX};

use crate::utils::{center, Message, Page};

pub struct LoadError(String);

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred while loading: {}", self.0)
    }
}

impl<E: std::error::Error> From<E> for LoadError {
    fn from(value: E) -> Self {
        Self(value.to_string())
    }
}

pub struct LoadingScreen {
    handle: JoinHandle<Result<Message, LoadError>>,
    state: ThrobberState,
    last_tick: Instant,
}

impl LoadingScreen {
    pub fn load<F, E>(func: F) -> Self
    where
        F: FnOnce() -> Result<Message, E> + Send + 'static,
        E: Into<LoadError> + Send + 'static,
    {
        let wrapper = move || func().map_err(|e| e.into());
        Self {
            handle: std::thread::spawn(wrapper),
            state: ThrobberState::default(),
            last_tick: Instant::now(),
        }
    }

    pub fn join(self) -> Message {
        return match self.handle.join() {
            Ok(Ok(message)) => message,
            Ok(Err(error)) => todo!("Show error screen: {error}"),
            Err(handle_error) => todo!("Show error scrren: {handle_error:?}"),
        };
    }

    fn tick(&mut self) {
        self.state.calc_next();
        self.last_tick = Instant::now();
    }
}

impl Page for LoadingScreen {
    fn render(&mut self, frame: &mut ratatui::Frame, area: ratatui::prelude::Rect) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let throbber = Throbber::default()
            .label("Loading words...")
            .throbber_style(
                Style::default()
                    .fg(ratatui::style::Color::Yellow)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(BRAILLE_SIX)
            .use_type(WhichUse::Spin);

        frame.render_stateful_widget(throbber, center, &mut self.state);
    }

    fn poll(&mut self) -> Option<Message> {
        if self.last_tick.elapsed() > Duration::from_millis(200) {
            self.tick();
        }
        if self.handle.is_finished() {
            return Some(Message::ShowLoaded);
        }

        None
    }
}
