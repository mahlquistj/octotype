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
    page::{self, Loading},
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
        if self.fetch_buffer.is_none() {
            if let Some(new_text) = self.mode.source.try_fetch()? {
                self.fetch_buffer = Some(new_text);
            } else if self.gladius_session.is_fully_typed() {
                return Err(FetchError::SourceError(
                    "Source fetched too slowly".to_string(),
                ));
            }
        } else if self.gladius_session.completion_percentage() > 50.0 {
            let Some(text) = self.fetch_buffer.take() else {
                unreachable!("Text in buffer was None after checking!");
            };
            self.gladius_session.push_string(&text);
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
                            let text = config.settings.theme.cursor.text;
                            let cursor = config.settings.theme.cursor.color;
                            style = style.bg(cursor).fg(text);
                        }

                        Span::from(ctx.character.char.to_string()).style(style)
                    })
                    .collect::<Line>();

                Some(rendered)
            },
            LineRenderConfig::new(area.width as usize).with_newline_breaking(true),
        );

        let area = center(area, Constraint::Percentage(80), Constraint::Percentage(80));
        let height = height_of_lines(&lines, area);

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(centered_padding(area, Some(height), None)))
            .centered();

        frame.render_widget(paragraph, area);
    }

    pub fn render_top(&self, _config: &Config) -> Option<Line<'_>> {
        let time = self.gladius_session.time_elapsed();
        let seconds = time.rem(60.0);
        let minutes = time / 60.0;

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

        Some(Line::raw(format!("{minutes:.0}:{seconds:0>2} {stats}")))
    }

    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        if self.gladius_session.is_fully_typed() {
            return Some(Message::Show(
                // TODO: Clone for now, but maybe we need a better solution for finalizing
                page::Stats::from(self.gladius_session.clone().finalize().unwrap()).into(),
            ));
        }

        if self.mode.conditions.words_typed.is_some()
            && let Err(error) = self.fetch_new_text(config)
        {
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
