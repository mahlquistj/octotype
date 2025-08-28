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

use crate::{config::Config, utils::Timestamp};
use crate::modes::ResolvedModeConfig;

mod stats;
mod text;

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

#[derive(Debug, thiserror::Error)]
#[error("Cannot create session with empty text")]
pub struct EmptySessionError;

/// Page: TypingSession
#[derive(Debug)]
pub struct TypingSession {
    text: Vec<Segment>,
    current_segment_idx: usize,
    first_keypress: Option<Instant>,
    stats: RunningStats,
    mode: Option<ResolvedModeConfig>,

    /// Caches
    current_error_cache: HashMap<usize, u16>,
    actual_error_cache: HashMap<usize, u16>,
    stat_cache: Option<StatsCache>,
    last_wpm_poll: Option<Instant>,
    last_input_len: usize,
}

impl Default for TypingSession {
    fn default() -> Self {
        Self {
            text: Vec::new(),
            current_segment_idx: 0,
            first_keypress: None,
            stats: RunningStats::default(),
            mode: None,
            current_error_cache: HashMap::new(),
            actual_error_cache: HashMap::new(),
            stat_cache: None,
            last_wpm_poll: None,
            last_input_len: 0,
        }
    }
}

impl TypingSession {
    /// Creates a new `TypingSession`
    pub fn new(text: Vec<Segment>) -> Result<Self, EmptySessionError> {
        if text.is_empty() {
            return Err(EmptySessionError);
        }

        Ok(Self {
            text,
            mode: None,
            ..Default::default()
        })
    }

    /// Get the current input length of all the segments
    fn input_length(&self) -> usize {
        self.text.iter().map(Segment::input_length).sum()
    }

    /// Get the current amount of errors in all of the segments
    fn get_current_errors(&mut self) -> u16 {
        let current_errors = self.current_segment().current_errors();
        self.current_error_cache
            .insert(self.current_segment_idx, current_errors);
        self.current_error_cache.values().sum()
    }

    /// Get the amount of actual errors (Corrected and uncorrected) in all of the segments
    fn get_actual_errors(&mut self) -> u16 {
        let actual_errors = self.current_segment().actual_errors();
        self.actual_error_cache
            .insert(self.current_segment_idx, actual_errors);
        self.actual_error_cache.values().sum()
    }

    /// Polls the session.
    ///
    /// Returns the stats if the session is done. Returns `None` otherwise
    fn poll_stats(&mut self) -> Option<Stats> {
        if self.text.iter().all(|seg| seg.is_done()) {
            let time = self.elapsed_minutes();
            let input_length = self.input_length();
            let wpm = self.calculate_wpm(time, input_length);
            let acc = self.calculate_acc();
            let stats = std::mem::take(&mut self.stats);
            return Some(stats.build_stats(&self.text, wpm, acc, time));
        }

        None
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
        let new = StatsCache {
            acc: self.calculate_acc(),
            wpm: with_wpm.then(|| {
                let input_len = self.input_length().abs_diff(self.last_input_len);
                self.calculate_wpm(time_since_poll, input_len)
            }),
        };

        let error = error.then_some(character);

        self.stats.update(time, new.acc, new.wpm, error, delete);

        match (&new.wpm, &mut self.stat_cache) {
            (None, Some(cached)) => cached.acc = new.acc,
            (_, _) => self.stat_cache = Some(new),
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
        if let Some(timestamp) = self.first_keypress {
            return timestamp.elapsed();
        }

        Duration::ZERO
    }

    /// Get the elapsed time of the session in minutes
    fn elapsed_minutes(&mut self) -> f64 {
        if let Some(timestamp) = self.first_keypress {
            return timestamp.elapsed().as_secs_f64() / 60.0;
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
}

// Rendering logic
impl TypingSession {
    pub fn render(&self, frame: &mut Frame, area: Rect, config: &Config) {
        // TODO: Find a better way to handle
        if self.current_segment_idx == self.text.len() {
            return;
        }

        let text = self
            .text
            .iter()
            .enumerate()
            .map(|(idx, seg)| seg.render_line(idx == self.current_segment_idx, &config.theme.text))
            .collect::<Vec<Line>>();

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let center =
            crate::utils::center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        frame.render_widget(paragraph.block(block), center);
    }

    pub fn render_top(&self, _config: &Config) -> Option<Line> {
        let time = self.elapsed();
        let seconds = time.as_secs().rem(60);
        let minutes = time.as_secs() / 60;
        Some(Line::raw(format!("{minutes}:{seconds:0>2} {}", {
            self.stat_cache
                .as_ref()
                .map_or_else(|| "".to_string(), |cache| cache.to_string())
        })))
    }

    // Mode support methods
    
    /// Creates a new `TypingSession` with mode configuration
    pub fn new_with_mode(text: Vec<Segment>, mode: ResolvedModeConfig) -> Result<Self, EmptySessionError> {
        let mut session = Self::new(text)?;
        session.mode = Some(mode);
        Ok(session)
    }
    
    /// Get the first keypress timestamp for mode completion checking
    pub fn get_first_keypress(&self) -> Option<Instant> {
        self.first_keypress
    }
    
    /// Helper methods for mode completion checking
    pub fn get_typed_word_count(&self) -> usize {
        self.text.iter()
            .map(|segment| segment.count_completed_words())
            .sum()
    }
    
    pub fn get_typed_char_count(&self) -> usize {
        self.text.iter()
            .map(|segment| segment.count_completed_chars())
            .sum()
    }
    
    pub fn calculate_accuracy(&mut self) -> f64 {
        let characters = self.input_length() as f64;
        let actual_errors = self.get_actual_errors() as f64;
        if characters == 0.0 {
            return 100.0;
        }
        (1.0 - (actual_errors / characters)) * 100.0
    }
    
    pub fn is_all_text_completed(&self) -> bool {
        self.text.iter().all(|segment| segment.is_done())
    }
    
    /// Check if session should end based on mode conditions
    pub fn should_end(&mut self) -> bool {
        if let Some(mode) = self.mode.clone() {
            mode.is_complete(self)
        } else {
            // Default behavior: end when all text is completed
            self.is_all_text_completed()
        }
    }

    pub fn poll(&mut self, _config: &Config) -> Option<Message> {
        self.poll_stats().map(|stats| Message::Show(stats.into()))
    }

    pub fn handle_events(
        &mut self,
        event: &crossterm::event::Event,
        _config: &Config,
    ) -> Option<Message> {
        if let Event::Key(key) = event {
            if key.is_press() {
                match key.code {
                    KeyCode::Char(character) => self.add(character),
                    KeyCode::Backspace => self.delete_input(),
                    _ => (),
                }
            }
        }

        None
    }
}
