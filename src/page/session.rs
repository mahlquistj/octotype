use std::{
    collections::HashMap,
    fmt::Display,
    ops::{Div, Rem},
    time::{Duration, Instant},
};

use crossterm::event::{Event, KeyCode};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph, Wrap},
};

use crate::{
    config::Config,
    utils::{Timestamp, fade},
};

mod mode;
mod stats;
mod text;

pub use mode::{CreateModeError, FetchError, Mode};
use stats::Wpm;
pub use stats::{RunningStats, Stats};
pub use text::Segment;

use super::Message;

/// Helper struct for caching stats throughout the session
#[derive(Debug)]
struct StatsCache {
    acc: f64,
    wpm: Option<Wpm>,
}

impl Display for StatsCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (raw, actual) = self.wpm.map_or(("?".to_string(), "?".to_string()), |wpm| {
            (wpm.raw.to_string(), wpm.actual.to_string())
        });
        let acc = self.acc;
        write!(f, "R: {raw:.2} | W: {actual:.2} | A: {acc:.2}%")
    }
}

/// Page: TypingSession
#[derive(Debug)]
pub struct TypingSession {
    text: Vec<Segment>,
    next_words: Option<Vec<Segment>>,
    stats: RunningStats,
    mode: Mode,

    // Trackers
    first_keypress: Option<Instant>,
    last_wpm_poll: Option<Instant>,
    last_input_len: usize,
    current_segment_idx: usize,

    // Caches
    current_error_cache: HashMap<usize, usize>,
    actual_error_cache: HashMap<usize, usize>,
    stat_cache: Option<StatsCache>,
}

impl TypingSession {
    /// Creates a new `TypingSession`
    pub fn new(config: &Config, mut mode: Mode) -> Result<Self, FetchError> {
        // TODO: Calculate segment size according to terminal size
        let text: Vec<Segment> = Self::text_to_segments(config, mode.source.fetch()?);
        let total_chars = text.iter().map(|seg| seg.len()).sum();

        Ok(Self {
            text,
            next_words: None,
            mode,
            stats: RunningStats::default(),
            first_keypress: None,
            last_wpm_poll: None,
            last_input_len: 0,
            current_segment_idx: 0,
            current_error_cache: HashMap::with_capacity(total_chars),
            actual_error_cache: HashMap::with_capacity(total_chars),
            stat_cache: None,
        })
    }

    /// Get the current input length of all the segments
    fn input_length(&self) -> usize {
        self.text.iter().map(Segment::input_length).sum()
    }

    /// Get the current amount of errors in all of the segments
    fn get_current_errors(&mut self) -> usize {
        let current_errors = self.current_segment().current_errors();
        self.current_error_cache
            .insert(self.current_segment_idx, current_errors);
        self.current_error_cache.values().sum()
    }

    /// Get the amount of actual errors (Corrected and uncorrected) in all of the segments
    fn get_actual_errors(&mut self) -> usize {
        let actual_errors = self.current_segment().actual_errors();
        self.actual_error_cache
            .insert(self.current_segment_idx, actual_errors);
        self.actual_error_cache.values().sum()
    }

    /// Polls the session.
    ///
    /// Returns the stats if the session is done. Returns `None` otherwise
    fn build_stats(&mut self) -> Stats {
        let time = self.elapsed_minutes();
        let input_length = self.input_length();
        let wpm = self.calculate_wpm(time, input_length);
        let acc = self.calculate_acc();
        let stats = std::mem::take(&mut self.stats);
        stats.build_stats(&self.text, wpm, acc, time)
    }

    /// Get the current active segment in the session
    fn current_segment(&mut self) -> &mut Segment {
        &mut self.text[self.current_segment_idx]
    }

    /// Update the stats with new data
    fn update_stats(&mut self, character: char, error: bool, delete: bool) {
        let time = self.elapsed_minutes();

        let time_since_poll = self
            .last_wpm_poll
            .map(|i| i.elapsed().as_secs_f64() / 60.0)
            .unwrap_or_default();

        let with_wpm = self.should_calc_wpm();
        let new_cache = StatsCache {
            acc: self.calculate_acc(),
            wpm: with_wpm.then(|| {
                let input_len = self.input_length().abs_diff(self.last_input_len);
                self.calculate_wpm(time_since_poll, input_len)
            }),
        };

        let error = error.then_some(character);

        self.stats
            .update(time, new_cache.acc, new_cache.wpm, error, delete);

        match (&new_cache.wpm, &mut self.stat_cache) {
            (None, Some(cached)) => cached.acc = new_cache.acc,
            (_, _) => self.stat_cache = Some(new_cache),
        };
    }

    /// Deletes a character from the session
    fn delete_input(&mut self) {
        let idx = self.current_segment_idx;
        let segment = self.current_segment();

        if segment.delete_input() {
            let char_idx = segment.input_length();
            let character = segment
                .get_char(char_idx + 1)
                .or_else(|| segment.get_char(char_idx))
                .expect("No character found"); // TODO: refactor this expect out
            return self.update_stats(character, false, true);
        }

        if idx > 0 {
            self.current_segment_idx -= 1;
            self.delete_input();
        }
    }

    /// Adds a character to the session
    fn add(&mut self, character: char) {
        let idx = self.current_segment_idx;
        let segment = self.current_segment();
        let actual_char = segment
            .get_char(segment.input_length())
            .expect("No character found"); // refactor
        // this expect out

        let is_error = !segment.add_input(character);

        if segment.is_done() && idx < (self.text.len() - 1) {
            self.current_segment_idx += 1;
        }

        self.update_stats(actual_char, is_error, false);
    }

    /// Get the elapsed `Duration` of the session
    fn elapsed(&self) -> Duration {
        let Some(timestamp) = self.first_keypress else {
            return Duration::ZERO;
        };

        timestamp.elapsed()
    }

    /// Get the elapsed time of the session in minutes
    fn elapsed_minutes(&mut self) -> f64 {
        if self.first_keypress.is_some() {
            return self.elapsed().as_secs_f64() / 60.0;
        }

        if self.input_length() > 0 {
            self.first_keypress = Some(Instant::now());
            self.last_wpm_poll = Some(Instant::now());
        }

        0.0
    }

    /// Checks if the session should calculate wpm
    fn should_calc_wpm(&self) -> bool {
        let Some(last_poll) = self.last_wpm_poll else {
            return false;
        };

        if last_poll.elapsed() > Duration::from_secs(1) {
            return true;
        }

        false
    }

    /// Calculates Wpm based on the given time and input_len
    fn calculate_wpm(&mut self, time: Timestamp, input_len: usize) -> Wpm {
        let frame_characters = input_len as f64;

        let raw = frame_characters.div(5.0).div(time);

        let current_errors = self.get_current_errors() as f64;

        let epm = current_errors.div(time);
        let actual = raw - epm;

        self.last_wpm_poll = Some(Instant::now());
        self.last_input_len = self.input_length();

        Wpm {
            raw,
            actual: actual.clamp(0.0, f64::MAX),
        }
    }

    /// Calculates the current accuracy of the session
    fn calculate_acc(&mut self) -> f64 {
        let characters = self.input_length() as f64;
        let actual_errors = self.get_actual_errors() as f64;
        (1.0 - (actual_errors / characters)) * 100.0
    }

    /// Get the first keypress timestamp for mode completion checking
    pub const fn get_first_keypress(&self) -> Option<Instant> {
        self.first_keypress
    }

    /// Helper methods for mode completion checking
    pub fn get_typed_word_count(&self) -> usize {
        self.text
            .iter()
            .map(|segment| segment.count_completed_words())
            .sum()
    }

    pub fn is_all_text_completed(&self) -> bool {
        self.text.iter().all(|segment| segment.is_done())
    }

    pub const fn is_text_almost_completed(&self) -> bool {
        self.current_segment_idx >= self.text.len() / 2
    }

    pub fn should_end(&mut self) -> bool {
        // End on error, if errors aren't allowed
        if !self.mode.conditions.allow_errors && self.get_current_errors() > 0 {
            return true;
        }

        // Time-based completion
        if let Some(time_limit) = self.mode.conditions.time
            && let Some(start_time) = self.get_first_keypress()
        {
            return start_time.elapsed() >= time_limit;
        }

        // Word count completion
        self.mode.conditions.words_typed.map_or_else(
            || self.is_all_text_completed(),
            |target_words| {
                let typed_words = self.get_typed_word_count();
                typed_words >= target_words as usize
            },
        )
    }

    fn text_to_segments(config: &Config, text: Vec<String>) -> Vec<Segment> {
        text.chunks(config.settings.words_per_line)
            .map(|words| {
                let string = words
                    .iter()
                    .cloned()
                    .map(|mut word| {
                        word.push(' ');
                        word
                    })
                    .collect::<String>();

                Segment::from_iter(string.chars())
            })
            .collect()
    }

    fn fetch_new_text(&mut self, config: &Config) -> Result<(), FetchError> {
        if self.next_words.is_none() {
            if let Some(new_text) = self.mode.source.try_fetch()? {
                let new_segments = Self::text_to_segments(config, new_text);
                self.next_words = Some(new_segments);
            } else if self.is_all_text_completed() {
                return Err(FetchError::SourceError(
                    "Source fetched too slowly".to_string(),
                ));
            }
        } else if self.is_text_almost_completed() {
            let Some(mut segments) = self.next_words.take() else {
                unreachable!("Segments are always Some");
            };
            self.text.append(&mut segments);
        }

        Ok(())
    }
}

// Rendering logic
impl TypingSession {
    pub fn render(&self, frame: &mut Frame, area: Rect, config: &Config) {
        // TODO: Find a better way to handle
        if self.current_segment_idx == self.text.len() {
            return;
        }

        let ghost_lines = config.settings.show_ghost_lines;
        let show_lines = if ghost_lines > 0 {
            (self.current_segment_idx.saturating_sub(ghost_lines))
                ..((self.current_segment_idx + ghost_lines).clamp(0, self.text.len()))
        } else {
            0..self.text.len()
        };
        let text_theme = &config.settings.theme.text;
        let term_bg = config.settings.theme.term_bg;
        let term_fg = config.settings.theme.term_fg;
        let ghost_fade_disabled = config.settings.disable_ghost_fade;

        let mut lines = Vec::with_capacity(show_lines.len());

        self.text
            .iter()
            .enumerate()
            .filter(|(idx, _)| show_lines.contains(idx))
            .for_each(|(idx, seg)| {
                let relative_idx = self.current_segment_idx.abs_diff(idx);
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
                let line = seg.render_line(
                    idx == self.current_segment_idx,
                    config,
                    success,
                    warning,
                    error,
                    foreground,
                );

                lines.push(line)
            });

        let paragraph = Paragraph::new(lines)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let center =
            crate::utils::center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        frame.render_widget(paragraph.block(block).alignment(Alignment::Center), center);
    }

    pub fn render_top(&self, _config: &Config) -> Option<Line<'_>> {
        let time = self.elapsed();
        let seconds = time.as_secs().rem(60);
        let minutes = time.as_secs() / 60;
        Some(Line::raw(format!("{minutes}:{seconds:0>2} {}", {
            self.stat_cache
                .as_ref()
                .map_or_else(|| "".to_string(), |cache| cache.to_string())
        })))
    }

    pub fn poll(&mut self, config: &Config) -> Option<Message> {
        if self.should_end() {
            return Some(Message::Show(self.build_stats().into()));
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
                KeyCode::Char(character) => self.add(character),
                KeyCode::Backspace if self.mode.conditions.allow_deletions => self.delete_input(),
                _ => (),
            }
        }

        None
    }
}
