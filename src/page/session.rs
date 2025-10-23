use std::ops::Rem;

use crossterm::event::{Event, KeyCode};
use derive_more::Display;
use gladius::{State, TypingSession, render::LineRenderConfig};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    prelude::Color,
    style::{Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Gauge, Paragraph, Wrap},
};

use crate::{
    config::Config,
    page::{self},
    utils::{center, centered_padding, fade, height_of_lines},
};

mod mode;

pub use mode::{CreateModeError, FetchError, Mode};

use super::Message;

const MIN_GAUGE_HEIGHT: u16 = 1;
const MAX_GAUGE_HEIGHT: u16 = 3;

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

        let [_, text_area, gauges_area] = Layout::vertical([
            Constraint::Percentage(20),
            Constraint::Percentage(60),
            Constraint::Percentage(20),
        ])
        .areas(area);
        let text_area = center(
            text_area,
            Constraint::Percentage(80),
            Constraint::Percentage(100),
        );

        let mut longest_line = 0;
        let lines = self.gladius_session.render_lines(
            |line| {
                let relative_idx = line.active_line_offset.unsigned_abs();
                if relative_idx > config.settings.show_ghost_lines {
                    return None;
                }

                longest_line = longest_line.max(line.contents.len());

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
            LineRenderConfig::new(text_area.width as usize).with_newline_breaking(true),
        );

        let height = height_of_lines(&lines, text_area);
        let padding = centered_padding(text_area, Some(height), Some(longest_line as u16));

        // Set cursor position if we found one
        if let Some((cursor_x, cursor_y)) = cursor_position {
            let cursor_area_x = text_area.x + padding.left + cursor_x;
            let cursor_area_y = text_area.y + padding.top + cursor_y;
            frame.set_cursor_position((cursor_area_x, cursor_area_y));
        }

        let paragraph = Paragraph::new(lines)
            .wrap(Wrap { trim: false })
            .block(Block::new().padding(padding));

        frame.render_widget(paragraph, text_area);

        self.render_gauges(config, frame, gauges_area);
    }

    pub fn render_gauges(&self, config: &Config, frame: &mut Frame, area: Rect) {
        let gauges = [
            self.mode.conditions.time.as_ref().map(|max| {
                let max = max.as_secs_f64();
                let elapsed = self.gladius_session.time_elapsed();

                let ratio = (elapsed / max).clamp(0.0, 1.0);
                let percent = (ratio * 100.0).round() as u16;

                let fg = match percent {
                    60..=80 => config.settings.theme.text.warning,
                    81..=100 => config.settings.theme.text.error,
                    _ => config.settings.theme.text.success,
                };

                Gauge::default()
                    .label(format!(
                        "Time: {}/{}",
                        format_time(elapsed),
                        format_time(max)
                    ))
                    .percent(percent)
                    .gauge_style(fg)
            }),
            self.mode.conditions.words_typed.as_ref().map(|goal| {
                let words_typed = self.gladius_session.words_typed_count();
                let percent = (words_typed.saturating_mul(100) + goal / 2) / goal;

                Gauge::default()
                    .label(format!("Words: {words_typed}/{goal}"))
                    .percent(percent.clamp(0, 100) as u16)
                    .gauge_style(config.settings.theme.text.highlight)
            }),
        ];

        let to_render = gauges.into_iter().flatten().collect::<Vec<_>>();
        if to_render.is_empty() {
            return;
        }

        let constraints = gauge_constraints(area, to_render.len());
        let areas = Layout::vertical(constraints).split(area);

        for (gauge, rect) in to_render.into_iter().zip(areas.iter()) {
            frame.render_widget(gauge, *rect);
        }
    }

    pub fn render_top(&self, _config: &Config) -> Option<Line<'_>> {
        let time = format_time(self.gladius_session.time_elapsed());

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
            .unwrap_or_default();

        Some(Line::raw(format!("{time} {stats}")))
    }

    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        if self.should_end() {
            let statistics = self.gladius_session.clone().finalize();

            // Save statistics if enabled
            if let Some(stats_manager) = &config.statistics_manager
                && let Err(error) = stats_manager.save_session(
                    &self.mode,
                    self.mode.mode_name.clone(),
                    self.mode.source_name.clone(),
                    &statistics,
                )
            {
                return Some(Message::Error(Box::new(error)));
            }

            return Some(Message::Show(page::Stats::from(statistics).into()));
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

#[derive(Display)]
#[display("{minutes}:{seconds}")]
struct Time {
    seconds: u16,
    minutes: u16,
}

fn format_time(time: f64) -> Time {
    Time {
        seconds: time.rem(60.0).trunc() as u16,
        minutes: (time / 60.0).trunc() as u16,
    }
}

fn gauge_constraints(area: Rect, desired_count: usize) -> Vec<Constraint> {
    if MIN_GAUGE_HEIGHT == 0 || MIN_GAUGE_HEIGHT > MAX_GAUGE_HEIGHT || area.height == 0 {
        return Vec::new();
    }

    // Fit as many as possible at the minimum height.
    let max_by_area = (area.height / MIN_GAUGE_HEIGHT) as usize;
    let n = desired_count.min(max_by_area);
    if n == 0 {
        return Vec::new();
    }

    // Start everyone at MIN height.
    let mut heights = vec![MIN_GAUGE_HEIGHT; n];

    // Spread any extra height as evenly as possible, capped by MAX per gauge.
    let used_min = MIN_GAUGE_HEIGHT * n as u16;
    let mut extra = area.height.saturating_sub(used_min);
    let per_gauge_cap = MAX_GAUGE_HEIGHT.saturating_sub(MIN_GAUGE_HEIGHT);
    let total_cap = per_gauge_cap.saturating_mul(n as u16);
    extra = extra.min(total_cap);

    if per_gauge_cap > 0 && extra > 0 {
        let base = extra / n as u16;
        let rem = extra % n as u16;
        for (i, h) in heights.iter_mut().enumerate() {
            let mut add = base;
            if (i as u16) < rem {
                add += 1;
            }
            *h = (*h + add).min(MAX_GAUGE_HEIGHT);
        }
    }

    heights.into_iter().map(Constraint::Length).collect()
}
