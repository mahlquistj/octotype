use std::fmt::Display;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    text::{Line, ToLine},
    widgets::{Block, Padding, Paragraph, Wrap},
};

use crate::{config::Config, utils::center};

use super::Message;

/// Page: Error
///
/// Displays an error
///
pub struct Error(String);

impl<E: Display> From<E> for Error {
    fn from(value: E) -> Self {
        Self(value.to_string())
    }
}

/// Rendering logic
impl Error {
    pub fn render(&self, frame: &mut ratatui::Frame, area: Rect, config: &Config) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

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

        let text = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(Padding::new(0, 0, center.height / 2, 0)));

        frame.render_widget(text, center);
    }

    pub fn render_top(&self, _config: &crate::config::Config) -> Option<Line> {
        Some(Line::from("<Enter> to return to menu"))
    }

    pub fn handle_events(&self, event: &Event, _config: &crate::config::Config) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            return match key.code {
                KeyCode::Enter => None, // TODO: Return to menu - need to pass mode configs
                _ => None,
            };
        };

        None
    }
}
