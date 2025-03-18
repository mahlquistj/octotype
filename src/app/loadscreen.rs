use std::{
    fmt::Display,
    thread::JoinHandle,
    time::{Duration, Instant},
};

use ratatui::{
    layout::{Alignment, Constraint},
    style::{Style, Stylize},
    widgets::{Block, Padding, Paragraph},
};
use throbber_widgets_tui::{Throbber, ThrobberState, WhichUse};

use crate::{
    config::Config,
    utils::{center, Message, Page},
};

/// An error during loading
#[derive(Debug)]
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

/// Page: LoadingScreen
pub struct LoadingScreen {
    /// The handle of the underlying thread
    handle: Option<JoinHandle<Result<Message, LoadError>>>,
    state: ThrobberState,
    last_tick: Instant,
}

impl LoadingScreen {
    /// Creats a new `LoadingScreen`.
    ///
    /// * `F`: The closure to run in the background
    /// * `E`: The error type returned by the closure (`F`)
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

    /// Checks if the underlying thread is finished
    fn is_finished(&self) -> bool {
        if let Some(handle) = &self.handle {
            return handle.is_finished();
        }

        false
    }

    /// Ticks the spinner
    fn tick(&mut self) {
        if self.last_tick.elapsed() > Duration::from_millis(200) {
            self.state.calc_next();
            self.last_tick = Instant::now();
        }
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
            .label("Loading...")
            .throbber_style(Style::default().fg(config.theme.spinner_color).bold())
            .throbber_set(config.theme.spinner_symbol.as_set())
            .use_type(WhichUse::Spin);

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        let text = Paragraph::new(throbber.to_line(&self.state))
            .alignment(Alignment::Center)
            .block(block);

        frame.render_widget(text, center);
    }

    fn poll(&mut self, _config: &Config) -> Option<Message> {
        self.tick();

        if !self.is_finished() {
            return None;
        }

        self.handle.take().unwrap().join().unwrap().ok() // TODO: Errors
    }
}
