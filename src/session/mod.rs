use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};
use stats::{GraphPoint, Wpm};
pub use stats::{RunningStats, Stats};
use std::{collections::HashMap, time::Instant};

mod library;
mod stats;
mod text;

pub use text::Segment;

pub use library::Library;

#[derive(Default)]
pub struct TypingSession {
    text: Vec<Segment>,
    current_segment_idx: usize,
    first_keypress: Option<Instant>,
    stats: RunningStats,
    error_cache: HashMap<usize, u16>,
    stat_cache: Option<GraphPoint>,
}

impl TypingSession {
    pub fn new(text: Vec<Segment>) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    pub fn length(&self) -> usize {
        self.text.iter().map(Segment::length).sum()
    }

    fn input_length(&self) -> usize {
        self.text.iter().map(Segment::input_length).sum()
    }

    fn get_errors(&mut self) -> u16 {
        let current_errors = self.current_segment().current_errors();
        self.error_cache
            .insert(self.current_segment_idx, current_errors);
        self.error_cache.values().sum()
    }

    pub fn poll(&self) -> Option<Stats> {
        if self.text.iter().all(|seg| seg.is_done()) {
            return Some(self.stats.build_stats(&self.text));
        }

        None
    }

    fn current_segment(&mut self) -> &mut Segment {
        &mut self.text[self.current_segment_idx]
    }

    fn update_stats(&mut self, character: char, error: bool, delete: bool) {
        let time = self.elapsed_minutes().unwrap_or_default();
        let point = self.calculate_stat_point(error.then(|| character));
        self.stat_cache = Some(point);
        self.stats.update(time, point, delete)
    }

    pub fn delete_input(&mut self) {
        let idx = self.current_segment_idx;
        let segment = self.current_segment();

        if segment.delete_input() {
            let character = segment
                .get_char(segment.input_length() + 1)
                .expect("No character found"); // refactor this expect out
            return self.update_stats(character, false, true);
        }

        if idx > 0 {
            self.current_segment_idx -= 1;
            self.delete_input();
        }
    }

    pub fn add(&mut self, character: char) {
        if self.first_keypress.is_none() {
            self.first_keypress = Some(Instant::now())
        }

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

    pub fn elapsed_minutes(&self) -> Option<f32> {
        self.first_keypress
            .as_ref()
            .map(|inst| inst.elapsed().as_secs_f32() / 60.0)
    }

    pub fn calculate_stat_point(&mut self, error: Option<char>) -> GraphPoint {
        let minutes = self.elapsed_minutes().unwrap_or_default();
        let characters = self.input_length() as f32;
        let raw = (characters / 5.0) / minutes;

        let errors = self.get_errors() as f32;
        // let actual = raw - (errors / minutes); // TODO: Why does this go negative?

        let actual = ((characters / 5.0) - errors) / minutes;

        let wpm = Wpm { raw, actual };
        let acc = 1.0 - (errors / characters); // Invert the result, as we want the actual
                                               // accuraccy, and not the percentage of errors

        GraphPoint {
            time: minutes,
            wpm,
            error,
            acc,
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> std::io::Result<()> {
        // TODO: Find a better way to handle
        if self.current_segment_idx == self.text.len() {
            return Ok(());
        }

        let [stats, words] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(2)]).areas(area);

        let stats_text = Line::raw(format!(
            "Elapsed: {:.2} {}",
            self.elapsed_minutes().unwrap_or_default(),
            {
                if let Some(point) = self.stat_cache {
                    format!(
                        "| Raw: {:.2} Wpm | Actual: {:.2} | Acc: {:.2}",
                        point.wpm.raw, point.wpm.actual, point.acc
                    )
                } else {
                    "".to_string()
                }
            },
        ));

        frame.render_widget(stats_text, stats);

        let text = self
            .text
            .iter()
            .enumerate()
            .map(|(idx, seg)| seg.render_line(idx == self.current_segment_idx))
            .collect::<Vec<Line>>();

        let paragraph = Paragraph::new(text)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let center = crate::utils::center(
            words,
            Constraint::Percentage(80),
            Constraint::Percentage(80),
        );

        let block = Block::new().padding(Padding::new(0, 0, center.height / 2, 0));

        frame.render_widget(paragraph.block(block), center);

        Ok(())
    }
}
