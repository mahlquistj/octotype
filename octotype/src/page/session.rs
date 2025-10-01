use std::ops::Rem;

use crossterm::event::{Event, KeyCode};
use gladius::{State, TypingSession, render::LineRenderConfig};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    prelude::Color,
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
    pub fn new(_config: &Config, mut mode: Mode) -> Result<Self, FetchError> {
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
    fn fetch_new_text(&mut self) -> Result<(), FetchError> {
        // As long as we don't have enough words to meet the conditions, keep trying to fetch
        if let Some(target) = self.mode.conditions.words_typed
            && target > self.gladius_session.word_count()
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

    fn should_end(&self) -> bool {
        if self.gladius_session.is_fully_typed() {
            return true;
        }

        if let Some(target) = self.mode.conditions.words_typed {
            return self.gladius_session.words_typed_count() == target;
        }

        if let Some(max_time) = self.mode.conditions.time {
            return self.gladius_session.time_elapsed() > max_time.as_secs_f64();
        }

        if !self.mode.conditions.allow_errors {
            return self.gladius_session.statistics().counters.errors > 0;
        }

        false
    }
}

// Rendering logic
impl Session {
    pub fn render(&self, frame: &mut Frame, area: Rect, config: &Config) {
        let mut cursor_position: Option<(u16, u16)> = None;
        let mut current_line = 0u16;

        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let lines = self.gladius_session.render_lines(
            |line| {
                let relative_idx = line.active_line_offset.unsigned_abs();
                if relative_idx > config.settings.show_ghost_lines {
                    return None;
                }

                let (success, warning, error, foreground) =
                    create_line_text_colors(relative_idx, config);

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

        let height = height_of_lines(&lines, area);
        let padding = centered_padding(area, Some(height), None);

        // Set cursor position if we found one
        if let Some((cursor_x, cursor_y)) = cursor_position {
            let cursor_area_x = area.x + padding.left + cursor_x;
            let cursor_area_y = area.y + padding.top + cursor_y;
            frame.set_cursor_position((cursor_area_x, cursor_area_y));
        }

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(padding));

        frame.render_widget(paragraph, area);
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

    pub fn poll(&mut self, _config: &Config) -> Option<Message> {
        if self.should_end() {
            return Some(Message::Show(
                // TODO: Clone for now, but we need a better solution for finalizing
                page::Stats::from(self.gladius_session.clone().finalize()).into(),
            ));
        }

        if let Err(error) = self.fetch_new_text() {
            return Some(Message::Error(Box::new(error)));
        }

        None
    }

    pub fn handle_events(&mut self, event: &Event, _config: &Config) -> Option<Message> {
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

fn create_line_text_colors(relative_idx: usize, config: &Config) -> (Color, Color, Color, Color) {
    let theme = &config.settings.theme;
    if config.settings.disable_ghost_fade || relative_idx == 0 {
        (
            theme.text.success,
            theme.text.warning,
            theme.text.error,
            theme.term_fg,
        )
    } else {
        let fade_percent = config.settings.ghost_opacity[relative_idx - 1];
        (
            fade(theme.text.success, theme.term_bg, fade_percent, false),
            fade(theme.text.warning, theme.term_bg, fade_percent, false),
            fade(theme.text.error, theme.term_bg, fade_percent, false),
            fade(theme.term_fg, theme.term_bg, fade_percent, false),
        )
    }
}
