use std::{char, cmp::Ordering, collections::HashSet};

use ratatui::{
    style::{Color, Modifier, Style, Stylize},
    text::ToSpan,
};

use crate::config::theme::TextTheme;

/// A result from an input
#[derive(Debug)]
pub enum CharacterResult {
    /// The character didn't match the actual character in the text
    Wrong,

    /// The character was wrong before, but was now corrected
    Corrected,

    /// The character was correct
    Right,
}

impl CharacterResult {
    /// Returns true if the characterresult is of the variant `Self::Wrong`
    const fn is_wrong(&self) -> bool {
        matches!(self, Self::Wrong)
    }
}

/// A segment of text, containing words.
#[derive(Default, Debug)]
pub struct Segment {
    tokens: Vec<char>,
    input: Vec<CharacterResult>,
    words: Vec<(usize, usize)>,
    wrong_inputs: HashSet<usize>,
    current_errors: u16,
}

impl Segment {
    /// Returns true if the segment is done (Input length matches text length)
    pub const fn is_done(&self) -> bool {
        self.input.len() == self.tokens.len()
    }

    /// Returns the current errors in the segment
    pub const fn current_errors(&self) -> u16 {
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
                (CharacterResult::Wrong, false)
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
    pub const fn input_length(&self) -> usize {
        self.input.len()
    }

    fn get_word(&self, char_idx: usize) -> Option<(usize, usize)> {
        let idx = self
            .words
            .binary_search_by(|range| {
                if range.0 > char_idx {
                    return Ordering::Greater;
                }

                if range.1 < char_idx {
                    return Ordering::Less;
                }

                Ordering::Equal // The number is within range
            })
            .ok()?;

        Some(self.words[idx])
    }

    fn is_word_misspelled(&self, word: (usize, usize)) -> bool {
        if self.input.is_empty() {
            return false;
        }

        let max_idx = self.input.len() - 1;

        let (start, end) = word;

        if start > max_idx {
            return false;
        }

        let range = start..=end.clamp(0, max_idx);

        let word = &self.input[range];
        word.iter().any(CharacterResult::is_wrong)
    }

    // Mode support methods

    pub fn count_completed_words(&self) -> usize {
        // Count words that have been fully typed
        let input_len = self.input.len();
        if input_len == 0 {
            return 0;
        }

        self.words
            .iter()
            .filter(|(_start, end)| *end < input_len)
            .count()
    }

    pub const fn count_completed_chars(&self) -> usize {
        // Count characters that have been typed (correctly or incorrectly)
        self.input.len()
    }
    
    pub const fn count_total_chars(&self) -> usize {
        self.tokens.len()
    }

    pub fn count_errors(&self) -> usize {
        // Count typing errors in this segment
        self.wrong_inputs.len()
    }

    /// Renders the segment as a `ratatui::Line`
    pub fn render_line(&self, show_cursor: bool, colors: &TextTheme) -> ratatui::prelude::Line<'_> {
        let mut current_word = None;
        let mut is_misspelled = false;
        let mut last_errors = 0;

        self.tokens
            .iter()
            .enumerate()
            .map(|(idx, character)| {
                let is_space = *character == ' ';

                match (&mut current_word, is_space) {
                    (None, false) => current_word = self.get_word(idx),
                    (Some(_), true) => current_word = None,
                    _ => (),
                };

                // Only re-check spelling if the error-count changes
                if last_errors != self.current_errors {
                    last_errors = self.current_errors;
                }

                is_misspelled = current_word
                    .as_ref()
                    .map(|&word| self.is_word_misspelled(word))
                    .unwrap_or_default();

                let mut style = Style::new();

                if let Some(c) = self.input.get(idx) {
                    style = match c {
                        CharacterResult::Right => style.fg(colors.success),
                        CharacterResult::Corrected => style.fg(colors.warning),
                        CharacterResult::Wrong => {
                            if is_space {
                                style.bg(colors.error)
                            } else {
                                style.fg(colors.error)
                            }
                        }
                    }
                    .add_modifier(Modifier::BOLD)
                };

                if is_misspelled {
                    style = style.underlined().underline_color(colors.error);
                }

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
        let tokens = iter.into_iter().collect::<Vec<char>>();
        let mut words = vec![];

        let mut start_of_word = 0; // First word always start at Zero
        tokens.iter().enumerate().for_each(|(idx, c)| {
            if *c == ' ' {
                words.push((start_of_word, idx - 1));
                start_of_word = idx + 1; // Set next start at next character
            }
        });

        Self {
            tokens,
            words,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use super::Segment;

    fn create_test_text() -> Segment {
        "This is a test, not an actual text ".chars().collect()
    }

    #[test]
    fn spellcheck() {
        let mut segment = create_test_text();

        for _ in 0..segment.tokens.len() {
            segment.add_input('x'); // add wrong input
        }

        println!("{:?}", segment.input);

        assert!(
            segment
                .words
                .iter()
                .all(|&word| segment.is_word_misspelled(word))
        )
    }

    #[test]
    fn word_detection() {
        let segment = create_test_text();

        println!("{:?}", segment.words);
        assert_eq!(segment.words.len(), 8);

        let mut words = segment.words.iter();

        assert_eq!(words.next().unwrap(), &(0, 3)); // This
        assert_eq!(words.next().unwrap(), &(5, 6)); // is
        assert_eq!(words.next().unwrap(), &(8, 8)); // a
        assert_eq!(words.next().unwrap(), &(10, 14)); // test,
        assert_eq!(words.next().unwrap(), &(16, 18)); // not
        assert_eq!(words.next().unwrap(), &(20, 21)); // an
        assert_eq!(words.next().unwrap(), &(23, 28)); // actual
        assert_eq!(words.next().unwrap(), &(30, 33)); // text
    }

    #[test]
    fn get_word() {
        let segment = create_test_text();
        assert_eq!(segment.get_word(2), Some((0, 3)));
    }
}
