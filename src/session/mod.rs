use core::f64;
use crossterm::event::{Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use stats::Wpm;
pub use stats::{RunningStats, Stats};
use std::{
    collections::HashMap,
    fmt::Display,
    ops::Div,
    time::{Duration, Instant},
};

mod stats;
mod text;

pub use text::Segment;

use crate::utils::{KeyEventHelper, Message, Page, Timestamp};

pub struct StatsCache {
    acc: f64,
    wpm: Option<Wpm>,
}

impl Display for StatsCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (raw, actual) = self.wpm.map_or(("?".to_string(), "?".to_string()), |wpm| {
            (wpm.raw.to_string(), wpm.actual.to_string())
        });
        let acc = self.acc;
        write!(f, "R: {raw} | W: {actual} | A: {acc}%")
    }
}

#[derive(Default)]
pub struct TypingSession {
    text: Vec<Segment>,
    current_segment_idx: usize,
    first_keypress: Option<Instant>,
    stats: RunningStats,
    current_error_cache: HashMap<usize, u16>,
    actual_error_cache: HashMap<usize, u16>,
    stat_cache: Option<StatsCache>,
    last_wpm_poll: Option<Instant>,
}

impl TypingSession {
    pub fn new(text: Vec<Segment>) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    fn input_length(&self) -> usize {
        self.text.iter().map(Segment::input_length).sum()
    }

    fn get_current_errors(&mut self) -> u16 {
        let current_errors = self.current_segment().current_errors();
        self.current_error_cache
            .insert(self.current_segment_idx, current_errors);
        self.current_error_cache.values().sum()
    }

    fn get_actual_errors(&mut self) -> u16 {
        let actual_errors = self.current_segment().actual_errors();
        self.actual_error_cache
            .insert(self.current_segment_idx, actual_errors);
        self.actual_error_cache.values().sum()
    }

    pub fn poll_stats(&mut self) -> Option<Box<Stats>> {
        if self.text.iter().all(|seg| seg.is_done()) {
            let time = self.elapsed_minutes();
            let final_point = self.calculate_stats(time, true);
            return Some(Box::new(self.stats.build_stats(
                &self.text,
                final_point.wpm?,
                final_point.acc,
            )));
        }

        None
    }

    fn current_segment(&mut self) -> &mut Segment {
        &mut self.text[self.current_segment_idx]
    }

    fn update_stats(&mut self, character: char, error: bool, delete: bool) {
        let time = self.elapsed_minutes();

        let with_wpm = self.should_calc_wpm();
        let new = self.calculate_stats(time, with_wpm);

        let error = error.then_some(character);

        self.stats.update(time, new.acc, new.wpm, error, delete);

        match (&new.wpm, &mut self.stat_cache) {
            (None, Some(cached)) => cached.acc = new.acc,
            (_, _) => self.stat_cache = Some(new),
        };
    }

    pub fn delete_input(&mut self) {
        let idx = self.current_segment_idx;
        let segment = self.current_segment();

        if segment.delete_input() {
            let char_idx = segment.input_length();
            let character = segment
                .get_char(char_idx + 1)
                .or(segment.get_char(char_idx))
                .expect("No character found"); // refactor this expect out
            return self.update_stats(character, false, true);
        }

        if idx > 0 {
            self.current_segment_idx -= 1;
            self.delete_input();
        }
    }

    pub fn add(&mut self, character: char) {
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

    pub fn elapsed_minutes(&mut self) -> f64 {
        if let Some(timestamp) = self.first_keypress {
            return timestamp.elapsed().as_secs_f64() / 60.0;
        }

        if self.input_length() > 0 {
            self.first_keypress = Some(Instant::now());
            self.last_wpm_poll = Some(Instant::now());
        }

        0.0
    }

    pub fn should_calc_wpm(&mut self) -> bool {
        let Some(time) = self.first_keypress else {
            return false;
        };

        let Some(last_poll) = self.last_wpm_poll else {
            return false;
        };

        if time.elapsed().abs_diff(last_poll.elapsed()) > Duration::from_secs(1) {
            return true;
        }

        false
    }

    pub fn calculate_stats(&mut self, time: Timestamp, with_wpm: bool) -> StatsCache {
        let characters = self.input_length() as f64;
        let actual_errors = self.get_actual_errors() as f64;

        let wpm = with_wpm.then(|| {
            let raw = characters.div(5.0).div(time);

            let current_errors = self.get_current_errors() as f64;

            let epm = current_errors.div(time);
            let actual = raw - epm;

            self.last_wpm_poll = Some(Instant::now());

            Wpm {
                raw,
                actual: actual.clamp(0.0, f64::MAX),
            }
        });

        let acc = (1.0 - (actual_errors / characters)) * 100.0;

        StatsCache { wpm, acc }
    }
}

impl Page for TypingSession {
    fn render(&mut self, frame: &mut Frame, area: Rect) {
        // TODO: Find a better way to handle
        if self.current_segment_idx == self.text.len() {
            return;
        }

        let text = self
            .text
            .iter()
            .enumerate()
            .map(|(idx, seg)| seg.render_line(idx == self.current_segment_idx))
            .collect::<Vec<Line>>();

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let center =
            crate::utils::center(area, Constraint::Percentage(80), Constraint::Percentage(80));

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        frame.render_widget(paragraph.block(block), center);
    }

    fn render_top(&mut self) -> Option<Line> {
        Some(Line::raw(format!("{:.2} {}", self.elapsed_minutes(), {
            if let Some(cache) = &self.stat_cache {
                cache.to_string()
            } else {
                "".to_string()
            }
        })))
    }

    fn handle_events(&mut self, event: &crossterm::event::Event) -> Option<Message> {
        if let Event::Key(key) = event {
            if key.is_press() {
                match key.code {
                    KeyCode::Char(character) => self.add(character),
                    KeyCode::Backspace => self.delete_input(),
                    _ => (),
                }
            }
        }

        if let Some(stats) = self.poll_stats() {
            return Some(Message::Show(stats));
        }

        None
    }
}
