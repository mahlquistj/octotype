use std::{char, collections::HashSet};

use ratatui::{
    style::{Color, Modifier, Style},
    text::ToSpan,
};

use crate::config::TextTheme;

/// A result from an input
#[derive(Debug)]
pub(crate) enum CharacterResult {
    /// The character didn't match the actual character in the text
    Wrong(char), // TODO: Use character here to display multiple wrong characters after a word, like monkeytype does.
    /// The character was wrong before, but was now corrected
    Corrected,
    /// The character was correct
    Right,
}

impl CharacterResult {
    /// Returns true if the characterresult is of the variant `Self::Wrong`
    fn is_wrong(&self) -> bool {
        matches!(self, Self::Wrong(_))
    }
}

/// A segment of text, containing words.
#[derive(Default, Debug)]
pub struct Segment {
    tokens: Vec<char>,
    input: Vec<CharacterResult>,
    wrong_inputs: HashSet<usize>,
    current_errors: u16,
}

impl Segment {
    /// Returns true if the segment is done (Input length matches text length)
    pub fn is_done(&self) -> bool {
        self.input.len() == self.tokens.len()
    }

    /// Returns the current errors in the segment
    pub fn current_errors(&self) -> u16 {
        self.current_errors
    }

    /// Returns the actual errors (Corrected and uncorrected) in the segment
    pub fn actual_errors(&self) -> u16 {
        self.wrong_inputs.len() as u16
    }

    /// Gets a character at specific index
    pub fn get_char(&self, idx: usize) -> Option<char> {
        self.tokens.get(idx).copied()
    }

    /// Adds to the segment input. Returns false if the input was incorrect
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

    /// Returns the current input length of the segment
    pub fn input_length(&self) -> usize {
        self.input.len()
    }

    /// Renders the segment as a `ratatui::Line`
    pub fn render_line(&self, show_cursor: bool, colors: &TextTheme) -> ratatui::prelude::Line<'_> {
        self.tokens
            .iter()
            .enumerate()
            .map(|(idx, character)| {
                let mut style = Style::new();
                if let Some(c) = self.input.get(idx) {
                    style = match c {
                        CharacterResult::Right => style.fg(colors.success),
                        CharacterResult::Corrected => style.fg(colors.warning),
                        CharacterResult::Wrong(_) => {
                            if *character == ' ' {
                                style.bg(colors.error)
                            } else {
                                style.fg(colors.error)
                            }
                        }
                    }
                    .add_modifier(Modifier::BOLD)
                };

                if show_cursor && idx == self.input.len() {
                    style = style.bg(Color::White).fg(Color::Black);
                }

                character.to_span().style(style)
            })
            .collect()
    }
}

// FromIterator impl to be able to use `.collect()`
impl FromIterator<char> for Segment {
    fn from_iter<T: IntoIterator<Item = char>>(iter: T) -> Self {
        let tokens = iter.into_iter().collect();
        Self {
            tokens,
            ..Default::default()
        }
    }
}
