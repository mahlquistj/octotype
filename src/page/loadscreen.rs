use std::{fmt::Display, thread::JoinHandle};

use ratatui::{
    layout::{Alignment, Constraint},
    text::{Line, ToSpan},
    widgets::{Block, Paragraph},
};

use crate::{
    config::{Config, theme::SpinnerState},
    utils::{center, centered_padding},
};

use super::Message;

/// An error during loading
#[derive(Debug)]
pub struct LoadError(String);

impl std::error::Error for LoadError {}

impl Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An error occurred while loading: {}", self.0)
    }
}

/// Page: Loading
pub struct Loading {
    /// The handle of the underlying thread
    handle: Option<JoinHandle<Result<Message, LoadError>>>,
    message: String,
    spinner_state: SpinnerState,
}

impl Loading {
    /// Creats a new `LoadingScreen`.
    ///
    /// * `F`: The closure to run in the background
    /// * `E`: The error type returned by the closure (`F`)
    pub fn load<F, E>(config: &Config, message: &str, func: F) -> Self
    where
        F: FnOnce() -> Result<Message, E> + Send + 'static,
        E: std::error::Error + Send + 'static,
    {
        let wrapper = move || func().map_err(|e| LoadError(e.to_string()));
        Self {
            handle: Some(std::thread::spawn(wrapper)),
            spinner_state: config.settings.theme.spinner.make_state(),
            message: message.to_string(),
        }
    }

    /// Checks if the underlying thread is finished
    fn is_finished(&self) -> bool {
        if let Some(handle) = &self.handle {
            return handle.is_finished();
        }

        false
    }

    fn join(&mut self) -> Result<Option<Message>, LoadError> {
        self.handle
            .take()
            .map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|_| Err(LoadError("Failed to join threadhandle".to_string())))
            })
            .transpose()
    }
}

// Rendering logic
impl Loading {
    pub fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &Config,
    ) {
        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let spinner = config
            .settings
            .theme
            .spinner
            .render(&mut self.spinner_state);
        let text = Line::from(vec![spinner, self.message.to_span()]);
        let height = (text.width() as u16).div_ceil(area.width);

        let text = Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(Block::new().padding(centered_padding(area, Some(height), None)));

        frame.render_widget(text, area);
    }

    pub fn poll(&mut self, _config: &Config) -> Option<Message> {
        self.spinner_state.tick();

        if !self.is_finished() {
            return None;
        }

        match self.join() {
            Ok(msg) => msg,
            Err(err) => Some(Message::Error(Box::new(err))),
        }
    }
}
