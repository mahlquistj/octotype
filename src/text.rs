use std::{char, collections::HashSet};

use ratatui::{
    style::{Color, Modifier, Style},
    text::{ToLine, ToSpan},
};

#[derive(Debug)]
enum CharacterResult {
    Wrong(char), // TODO: Use character here to display multiple wrong characters after a word, like monkeytype does.
    Corrected, // TODO: Support seeing if a character was typed wrong before, but is now corrected.
    Right,
}

#[derive(Default, Debug)]
pub struct Segment {
    tokens: Vec<char>,
    input: Vec<CharacterResult>,
    wrong_inputs: HashSet<usize>,
}

impl Segment {
    pub fn is_done(&self) -> bool {
        self.input.len() == self.tokens.len()
    }

    pub fn add_input(&mut self, character: char) {
        let current = self.input.len();
        let matches_current = self.tokens[current] == character;
        let was_wrong = self.wrong_inputs.contains(&current);

        self.input.push(match (matches_current, was_wrong) {
            (true, false) => CharacterResult::Right,
            (true, true) => CharacterResult::Corrected,
            (false, _) => {
                self.wrong_inputs.insert(current);
                CharacterResult::Wrong(character)
            }
        });
    }

    /// Deletes one char from the input. Returns false if input was empty.
    pub fn delete_input(&mut self) -> bool {
        self.input.pop().is_some()
    }

    pub fn length(&self) -> usize {
        self.tokens.len()
    }

    pub fn input_length(&self) -> usize {
        self.input.len()
    }

    pub fn render_line(&self, show_cursor: bool) -> ratatui::prelude::Line<'_> {
        self.tokens
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

                if show_cursor && idx == self.input.len() {
                    style = style.bg(Color::White).fg(Color::Black);
                }

                character.to_span().style(style)
            })
            .collect()
    }
}

impl FromIterator<char> for Segment {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let tokens = iter.into_iter().collect();
        Self {
            tokens,
            ..Default::default()
        }
    }
}
