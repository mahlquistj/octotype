use std::{
    fmt::Display,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use ratatui::{
    layout::{Alignment, Constraint},
    style::Style,
    widgets::{Block, Padding, Paragraph},
};
use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse, BRAILLE_SIX};

use crate::{
    config::Config,
    utils::{center, Message, Page},
};

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
    handle: Option<JoinHandle<Result<Message, LoadError>>>,
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
            handle: Some(std::thread::spawn(wrapper)),
            state: ThrobberState::default(),
            last_tick: Instant::now(),
        }
    }

    fn tick(&mut self) {
        self.state.calc_next();
        self.last_tick = Instant::now();
    }
}

impl Page for LoadingScreen {
    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let throbber = Throbber::default()
            .label("Loading words...")
            .throbber_style(
                Style::default()
                    .fg(config.theme.spinner)
                    .add_modifier(ratatui::style::Modifier::BOLD),
            )
            .throbber_set(BRAILLE_SIX)
            .use_type(WhichUse::Spin);

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        let text = Paragraph::new(throbber.to_line(&self.state))
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(text, center);
    }

    fn poll(&mut self, _config: &Config) -> Option<Message> {
        if self.last_tick.elapsed() > Duration::from_millis(200) {
            self.tick();
        }

        if let Some(handle) = &self.handle {
            if !handle.is_finished() {
                return None;
            }
        }

        let show = self.handle.take()?.join().ok()?.ok()?; // TODO: Errors

        Some(show)
    }
}
