use std::fmt::Display;

use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    text::{Line, ToLine},
    widgets::{Block, Padding, Paragraph, Wrap},
};

use crate::utils::{center, KeyEventHelper, Message, Page};

use super::Menu;

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

impl Page for Error {
    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &crate::config::Config,
    ) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let mut lines =
            vec![
                Line::styled("[Error]", Style::new().bold().fg(config.theme.text.error)).centered(),
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

    fn render_top(&mut self, _config: &crate::config::Config) -> Option<Line> {
        Some(Line::from("<Enter> to return to menu"))
    }

    fn handle_events(
        &mut self,
        event: &Event,
        _config: &crate::config::Config,
    ) -> Option<crate::utils::Message> {
        if let Event::Key(key) = event {
            if key.is_press() {
                return match key.code {
                    KeyCode::Enter => Some(Message::Show(Menu::new().boxed())),
                    _ => None,
                };
            };
        }

        None
    }
}
