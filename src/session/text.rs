use std::{char, collections::HashSet};

use ratatui::{
    style::{Color, Modifier, Style},
    text::ToSpan,
};

#[derive(Debug)]
pub(crate) enum CharacterResult {
    Wrong(char), // TODO: Use character here to display multiple wrong characters after a word, like monkeytype does.
    Corrected, // TODO: Support seeing if a character was typed wrong before, but is now corrected.
    Right,
}

impl CharacterResult {
    fn is_wrong(&self) -> bool {
        matches!(self, Self::Wrong(_))
    }
}

#[derive(Default, Debug)]
pub struct Segment {
    tokens: Vec<char>,
    pub(crate) input: Vec<CharacterResult>,
    wrong_inputs: HashSet<usize>,
    current_errors: u16,
}

impl Segment {
    pub fn is_done(&self) -> bool {
        self.input.len() == self.tokens.len()
    }

    pub fn current_errors(&self) -> u16 {
        self.current_errors
    }

    pub fn actual_errors(&self) -> u16 {
        self.wrong_inputs.len() as u16
    }

    pub fn get_char(&self, idx: usize) -> Option<char> {
        self.tokens.get(idx).copied()
    }

    /// Adds to this segments input. Returns false if the input was incorrect
    pub fn add_input(&mut self, character: char) -> bool {
        let current = self.input.len();
        let matches_current = self.tokens[current] == character;
        let was_wrong = self.wrong_inputs.contains(&current);

        // TODO: refactor bool return
        let (result, is_right) = match (matches_current, was_wrong) {
            (true, false) => (CharacterResult::Right, true),
            (true, true) => (CharacterResult::Corrected, true),
            (false, _) => {
                self.wrong_inputs.insert(current);
                self.current_errors += 1;
                (CharacterResult::Wrong(character), false)
            }
        };
        self.input.push(result);
        is_right
    }

    /// Deletes one char from the input. Returns false if input was empty.
    pub fn delete_input(&mut self) -> bool {
        if let Some(res) = self.input.pop() {
            if res.is_wrong() {
                self.current_errors -= 1;
            }
            return true;
        }

        false
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
