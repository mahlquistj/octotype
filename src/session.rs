use std::time::Instant;

use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, ToSpan},
    widgets::{Block, Padding, Paragraph, Wrap},
    Frame,
};

use crate::text::Segment;

#[derive(Default)]
pub struct TypingSession {
    text: Vec<Segment>,
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
        self.text.iter().map(|seg| seg.length()).sum()
    }

    pub fn is_done(&self) -> bool {
        self.text.iter().all(|seg| seg.is_done())
    }

    pub fn delete_input(&mut self) {
        todo!()
    }

    pub fn add(&mut self, characters: &[char]) {
        if self.first_keypress.is_none() {
            self.first_keypress = Some(Instant::now())
        }

        for &c in characters {
            todo!()
        }
    }

    pub fn calculate_wpm(&self) -> f64 {
        let minutes = self
            .first_keypress
            .map(|inst| inst.elapsed().as_secs_f64())
            .unwrap_or_default()
            / 60.0;
        let characters = self.length() as f64;
        (characters / 5.0) / minutes
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> std::io::Result<()> {
        let [stats, words] =
            Layout::vertical([Constraint::Length(1), Constraint::Fill(2)]).areas(area);

        let stats_text = Line::raw(format!("Raw: {:.2}", self.calculate_wpm()));

        frame.render_widget(stats_text, stats);

        let text = self
            .text
            .iter()
            .enumerate()
            .map(|(idx, character)| {
                let mut style = if let Some(c) = self.input.get(idx) {
                    match c {
                        CharacterResult::Right => Style::new().fg(Color::Green),
                        CharacterResult::Corrected => Style::new().fg(Color::Yellow),
                        CharacterResult::Wrong(_) => Style::new().fg(Color::Red),
                    }
                    .add_modifier(Modifier::BOLD)
                } else {
                    Style::new().fg(Color::White)
                };

                if idx == self.input.len() {
                    style = style.bg(Color::White).fg(Color::Black);
                }

                character.to_span().style(style)
            })
            .collect::<Line>();

        let paragraph = Paragraph::new(text).wrap(Wrap { trim: false });

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
