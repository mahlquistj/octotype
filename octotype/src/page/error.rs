use std::fmt::Display;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    text::{Line, ToLine},
    widgets::{Block, Paragraph, Wrap},
};

use crate::{
    config::Config,
    utils::{center, centered_padding, height_of_lines},
};

use super::Message;

/// Page: Error
///
/// Displays an error
///
pub struct Error(String);

impl Error {
    pub const fn new(error: String) -> Self {
        Self(error)
    }
}

impl<E: Display> From<E> for Error {
    fn from(value: E) -> Self {
        Self(value.to_string())
    }
}

/// Rendering logic
impl Error {
    pub fn render(&self, frame: &mut ratatui::Frame, area: Rect, config: &Config) {
        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let mut lines = vec![
            Line::styled(
                "[Error]",
                Style::new().bold().fg(config.settings.theme.text.error),
            )
            .centered(),
        ];

        let error_lines = self
            .0
            .split('\n')
            .map(str::to_string)
            .collect::<Vec<String>>();

        for line in &error_lines {
            lines.push(line.to_line().centered());
        }

        let height: u16 = height_of_lines(&lines, area);

        let text = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(centered_padding(area, Some(height), None)));

        frame.render_widget(text, area);
    }

    pub fn render_top(&self, _config: &crate::config::Config) -> Option<Line<'_>> {
        Some(Line::from("<Enter> to return to menu"))
    }

    pub fn handle_events(&self, event: &Event, _config: &crate::config::Config) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            return match key.code {
                KeyCode::Enter => Some(Message::Reset),
                _ => None,
            };
        };

        None
    }
}
