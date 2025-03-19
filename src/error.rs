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
pub struct Error(pub Box<dyn std::error::Error>);

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
        .block(Block::new().padding(Padding::new(0, 0, center.height / 2, 0)));

        frame.render_widget(text, center);
    }

    fn render_top(&mut self, _config: &crate::config::Config) -> Option<Line> {
        Some(Line::raw("ERROR"))
    }
}

impl<E: 'static + std::error::Error> From<E> for Error {
    fn from(value: E) -> Self {
        Self(Box::new(value))
    }
}
