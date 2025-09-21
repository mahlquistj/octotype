use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Div, Rem},
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode};
use gladius::{
    CharacterResult, State, TypingSession,
    render::{LineRenderConfig, RenderingContext},
};
use ratatui::{
    Frame,
    layout::{Constraint, Rect},
    style::{Modifier, Style, Stylize},
    text::{Line, Span, ToSpan},
    widgets::{Block, Paragraph, Wrap},
};

use crate::{
    config::Config,
    utils::{Timestamp, center, centered_padding, fade, height_of_lines},
};

mod mode;

pub use mode::{CreateModeError, FetchError, Mode};

use super::Message;

/// Page: TypingSession
#[derive(Debug)]
pub struct Session {
    gladius_session: TypingSession,
    next_words: Option<String>,
    mode: Mode,
}

impl Session {
    /// Creates a new `TypingSession`
    pub fn new(config: &Config, mut mode: Mode) -> Result<Self, FetchError> {
        let text = mode.source.fetch()?;
        let gladius_session = TypingSession::new(&text).expect("Failed to create TypingSession");

        Ok(Self {
            gladius_session,
            next_words: None,
            mode,
        })
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
                let relative_idx = line.active_line_offset.abs() as usize;
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

                        ctx.character.char.to_span().style(style)
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
        // TODO: SHOW STATS
        // Some(Line::raw(format!("{minutes:.0}:{seconds:0>2} {}", {
        //     self.stat_cache
        //         .as_ref()
        //         .map_or_else(|| "".to_string(), |cache| cache.to_string())
        // })))
        None
    }

    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        // if self.should_end() {
        //     return Some(Message::Show(self.build_stats().into()));
        // }
        //
        // if self.mode.conditions.words_typed.is_some()
        //     && let Err(error) = self.fetch_new_text(config)
        // {
        //     return Some(Message::Error(Box::new(error)));
        // }

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
