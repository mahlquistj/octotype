use std::ops::Rem;

use crossterm::event::{Event, KeyCode};
use gladius::{State, TypingSession, render::LineRenderConfig};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Wrap},
};

use crate::{
    config::Config,
    page::{self},
    utils::{center, centered_padding, fade, height_of_lines},
};

mod mode;

pub use mode::{CreateModeError, FetchError, Mode};

use super::Message;

/// Page: TypingSession
#[derive(Debug)]
pub struct Session {
    gladius_session: TypingSession,
    fetch_buffer: Option<String>,
    mode: Mode,
}

impl Session {
    /// Creates a new `TypingSession`
    pub fn new(config: &Config, mut mode: Mode) -> Result<Self, FetchError> {
        let text = mode.source.fetch()?;
        // Safety: Sources already check for empty output - This is the only error that can happen
        // when initializing a TypingSession
        let gladius_session = TypingSession::new(&text).expect("Failed to create TypingSession");

        Ok(Self {
            gladius_session,
            fetch_buffer: None,
            mode,
        })
    }
}

impl Session {
    fn fetch_new_text(&mut self, config: &Config) -> Result<(), FetchError> {
        // As long as we don't have enough words to meet the conditions, keep trying to fetch
        if let Some(target) = self.mode.conditions.words_typed
            && target as usize > self.gladius_session.word_count()
        {
            if self.fetch_buffer.is_none() {
                if let Some(new_text) = self.mode.source.try_fetch()? {
                    self.fetch_buffer = Some(new_text);
                } else if self.gladius_session.is_fully_typed() {
                    return Err(FetchError::SourceError(
                        "Source fetched too slowly".to_string(),
                    ));
                }
            } else if let Some(text) = self.fetch_buffer.take() {
                self.gladius_session.push_string(&text);
            }
        }
        Ok(())
    }
}

// Rendering logic
impl Session {
    pub fn render(&self, frame: &mut Frame, area: Rect, config: &Config) {
        let text_theme = &config.settings.theme.text;
        let term_bg = config.settings.theme.term_bg;
        let term_fg = config.settings.theme.term_fg;
        let ghost_fade_disabled = config.settings.disable_ghost_fade;

        let mut cursor_position: Option<(u16, u16)> = None;
        let mut current_line = 0u16;

        let lines = self.gladius_session.render_lines(
            |line| {
                let relative_idx = line.active_line_offset.unsigned_abs();
                if relative_idx > config.settings.show_ghost_lines {
                    return None;
                }

                let (success, warning, error, foreground) =
                    if ghost_fade_disabled || relative_idx == 0 {
                        (
                            text_theme.success,
                            text_theme.warning,
                            text_theme.error,
                            term_fg,
                        )
                    } else {
                        let fade_percent = config.settings.ghost_opacity[relative_idx - 1];
                        (
                            fade(text_theme.success, term_bg, fade_percent, false),
                            fade(text_theme.warning, term_bg, fade_percent, false),
                            fade(text_theme.error, term_bg, fade_percent, false),
                            fade(term_fg, term_bg, fade_percent, false),
                        )
                    };

                let mut current_col = 0u16;
                let rendered = line
                    .contents
                    .iter()
                    .map(|ctx| {
                        let mut style = Style::new().fg(foreground);
                        let is_space = ctx.character.char == ' ';

                        style = match ctx.character.state {
                            State::Correct => style.fg(success),
                            State::Corrected => style.fg(warning),
                            State::Wrong => {
                                if is_space {
                                    style.bg(error)
                                } else {
                                    style.fg(error)
                                }
                            }
                            _ => style,
                        }
                        .add_modifier(Modifier::BOLD);

                        if let Some(word) = ctx.word
                            && word.state == State::Wrong
                        {
                            style = style.underlined().underline_color(error);
                        }

                        if ctx.has_cursor {
                            // Position cursor at the current character
                            cursor_position = Some((current_col, current_line));
                        }

                        let span = Span::from(ctx.character.char.to_string()).style(style);
                        current_col += 1;
                        span
                    })
                    .collect::<Line>();

                current_line += 1;
                Some(rendered)
            },
            LineRenderConfig::new(area.width as usize).with_newline_breaking(true),
        );

        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        let height = height_of_lines(&lines, area);

        // Calculate line width before moving lines into paragraph
        let cursor_line_width = cursor_position.and_then(|(_, cursor_y)| {
            if cursor_y < lines.len() as u16 {
                Some(lines[cursor_y as usize].width() as u16)
            } else {
                None
            }
        });

        let padding = centered_padding(area, Some(height), None);
        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(padding))
            .centered();

        frame.render_widget(paragraph, area);

        // Set cursor position if we found one
        if let Some((cursor_x, cursor_y)) = cursor_position {
            // Calculate where the centered line starts within the content area
            let line_width = cursor_line_width.unwrap_or(0);
            let content_width = area.width.saturating_sub(padding.left + padding.right);
            let line_start_offset = content_width.saturating_sub(line_width) / 2;
            // Adjust for even line length centering (add 1 when line length is even)
            let centering_adjustment = if line_width % 2 == 0 { 1 } else { 0 };

            let cursor_area_x =
                area.x + padding.left + line_start_offset + centering_adjustment + cursor_x;
            let cursor_area_y = area.y + padding.top + cursor_y;
            frame.set_cursor_position((cursor_area_x, cursor_area_y));
        }
    }

    pub fn render_top(&self, _config: &Config) -> Option<Line<'_>> {
        let time = self.gladius_session.time_elapsed();
        let seconds = time.rem(60.0).trunc() as u8;
        let minutes = (time / 60.0).trunc() as u32;

        let stats = self
            .gladius_session
            .statistics()
            .measurements
            .last()
            .map(|measure| {
                format!(
                    "C: %{:.2} | W: {:.2} | A: {:2} | I: {:.2}",
                    measure.consistency.actual_percent,
                    measure.wpm.actual,
                    measure.accuracy.actual,
                    measure.ipm.actual
                )
            })
            .unwrap_or("".to_string());

        Some(Line::raw(format!("{minutes}:{seconds:0>2} {stats}")))
    }

    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        if self.gladius_session.is_fully_typed() {
            return Some(Message::Show(
                // TODO: Clone for now, but maybe we need a better solution for finalizing
                page::Stats::from(self.gladius_session.clone().finalize().unwrap()).into(),
            ));
        }

        if let Err(error) = self.fetch_new_text(config) {
            return Some(Message::Error(Box::new(error)));
        }

        None
    }

    pub fn handle_events(
        &mut self,
        event: &crossterm::event::Event,
        _config: &Config,
    ) -> Option<Message> {
        if let Event::Key(key) = event
            && key.is_press()
        {
            match key.code {
                KeyCode::Char(character) => {
                    self.gladius_session.input(Some(character));
                }
                KeyCode::Backspace if self.mode.conditions.allow_deletions => {
                    self.gladius_session.input(None);
                }
                _ => (),
            }
        }

        None
    }
}
