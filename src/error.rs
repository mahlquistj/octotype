use ratatui::{
    layout::Constraint,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph},
};

use crate::utils::{center, Page};

/// Page: Error
///
/// Displays an error
///
pub struct Error(Box<dyn std::error::Error>);

impl Page for Error {
    fn render(
        &mut self,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
        config: &crate::config::Config,
    ) {
        let center = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        let text = Paragraph::new(Line::from(vec![
            Span::styled("Error: ", Style::new().bold().fg(config.theme.text.error)),
            Span::raw(self.0.to_string()),
        ]))
        .block(Block::new().padding(Padding::new(
            0,
            0,
            center.height / 2,
            center.width / 4,
        )));

        frame.render_widget(text, center);
    }
}

impl<E: 'static + std::error::Error> From<E> for Error {
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}
