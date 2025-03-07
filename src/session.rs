use std::{
    fmt::format,
    time::{Duration, Instant},
};

use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    text::Line,
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use crate::text::Segment;

#[derive(Default)]
pub struct TypingSession {
    text: Vec<Segment>,
    current_segment_idx: usize,
    pub(crate) first_keypress: Option<Instant>,
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

    pub fn is_done(&self) -> bool {
        self.text.iter().all(|seg| seg.is_done())
    }

    fn current_segment(&mut self) -> &mut Segment {
        &mut self.text[self.current_segment_idx]
    }

    pub fn delete_input(&mut self) {
        if !self.current_segment().delete_input() && self.current_segment_idx > 0 {
            self.current_segment_idx -= 1;
            self.delete_input();
        }
    }

    pub fn add(&mut self, characters: &[char]) {
        if self.first_keypress.is_none() {
            self.first_keypress = Some(Instant::now())
        }

        for &character in characters {
            self.current_segment().add_input(character)
        }

        if self.current_segment().is_done() {
            self.current_segment_idx += 1;
        }
    }

    pub fn elapsed_minutes(&self) -> Option<f64> {
        self.first_keypress
            .as_ref()
            .map(|inst| inst.elapsed().as_secs_f64() / 60.0)
    }

    pub fn calculate_wpm(&self) -> f64 {
        let minutes = self.elapsed_minutes().unwrap_or_default();
        let characters = self.length() as f64;
        (characters / 5.0) / minutes
    }

    pub fn calculate_wpm_realtime(&self) -> f64 {
        let minutes = self.elapsed_minutes().unwrap_or_default();
        let characters = self.input_length() as f64;
        (characters / 5.0) / minutes
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> std::io::Result<()> {
        let [stats, words] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(2)]).areas(area);

        let stats_text = Line::raw(format!(
            "Elapsed: {:.2} | Raw: {:.2}",
            self.elapsed_minutes().unwrap_or_default(),
            self.calculate_wpm_realtime(),
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
