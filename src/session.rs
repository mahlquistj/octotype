use std::time::Instant;

use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, ToSpan},
    Frame,
};

#[derive(Debug)]
enum CharacterResult {
    Wrong(char), // TBD: Use character here to display multiple wrong characters after a word, like monkeytype does.
    Corrected,   // TBD: Support seeing if a character was typed wrong before, but is now corrected.
    Right,
}

#[derive(Default)]
pub struct TypingSession {
    text: Vec<char>,
    input: Vec<CharacterResult>,
    pub(crate) first_keypress: Option<Instant>,
}

impl TypingSession {
    pub fn new(text: Vec<char>) -> Self {
        Self {
            text,
            ..Default::default()
        }
    }

    pub fn length(&self) -> usize {
        self.text.len()
    }

    pub fn is_done(&self) -> bool {
        self.input.len() == self.text.len()
    }

    pub fn pop(&mut self) {
        self.input.pop();
    }

    pub fn add(&mut self, character: char) {
        if self.first_keypress.is_none() {
            self.first_keypress = Some(Instant::now())
        }

        let current = self.input.len();

        if self.text[current] == character {
            self.input.push(CharacterResult::Right);
        } else {
            self.input.push(CharacterResult::Wrong(character));
        }
    }

    pub fn render(&mut self, frame: &mut Frame, area: Rect) -> std::io::Result<()> {
        let line: Line = self
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
            .collect();

        let center = crate::utils::center(
            area,
            Constraint::Length(line.width() as u16),
            Constraint::Length(1),
        );

        frame.render_widget(line, center);
        Ok(())
    }
}
