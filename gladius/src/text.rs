use web_time::Instant;

use crate::{AVERAGE_WORD_LENGTH, TempStatistics};

#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    // == Pre delete or add ==
    /// The text has never been touched
    #[default]
    None,

    // == Post delete ==
    /// The text was wrong, but has since been deleted or corrected
    WasWrong,
    /// The text was corrected, but has since been deleted
    WasCorrected,
    /// The text was correct, but has since been deleted
    WasCorrect,

    // == Post add ==
    /// The text is wrong
    Wrong,
    /// The text was corrected
    Corrected,
    /// The text is correct
    Correct,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CharacterResult {
    Deleted(State),
    Wrong,
    Corrected,
    Correct,
}

pub struct Word {
    pub start: usize,
    pub end: usize,
    pub state: State,
}

pub struct Character {
    pub char: char,
    pub state: State,
}

pub struct Text {
    characters: Vec<Character>,
    words: Vec<Word>,
    input: Vec<char>,
    stats: TempStatistics,
    started_at: Option<Instant>,
}

impl Text {
    pub fn new(string: &str) -> Option<Self> {
        if string.is_empty() {
            return None;
        }

        let mut text = Self::with_capacity(string.len());
        text.push(string);

        Some(text)
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            characters: Vec::with_capacity(capacity),
            words: Vec::with_capacity(capacity / AVERAGE_WORD_LENGTH),
            input: Vec::with_capacity(capacity),
            stats: TempStatistics::default(),
            started_at: None,
        }
    }

    /// Returns the amount of characters currently in the [Text].
    pub fn text_len(&self) -> usize {
        self.characters.len()
    }

    /// Returns the current character awaiting input.
    pub fn current_character(&self) -> &Character {
        // Safety: It's impossible for the user to create an empty Text
        self.characters
            .get(self.input.len())
            .or_else(|| self.characters.last())
            .unwrap()
    }

    /// Returns true if the amount of characters currently in the [Text]'s input is 0.
    pub fn is_input_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Returns the amount of characters currently in the [Text]'s input.
    pub fn input_len(&self) -> usize {
        self.input.len()
    }

    /// Returns true if the text has been fully typed out.
    ///
    /// This means no additions or deletions will be accepted.
    pub fn is_fully_typed(&self) -> bool {
        self.input.len() == self.characters.len()
    }

    /// Get the current statistics recorded for the text
    pub fn statistics(&self) -> &TempStatistics {
        &self.stats
    }

    /// Push more characters to the [Text].
    ///
    /// This allows for dynamically adding text during typing.
    pub fn push(&mut self, string: &str) {
        let mut current_word_start: Option<usize> = None;
        string.char_indices().fold(self, |text, (index, char)| {
            // Find Word-indices and create Characters
            let is_whitespace = char.is_ascii_whitespace();

            if let Some(word_start) = current_word_start.take_if(|_| is_whitespace) {
                // Add new word, as we've hit whitespace
                text.words.push(Word {
                    start: word_start,
                    end: index,
                    state: State::default(),
                });
            } else if !is_whitespace {
                // Start tracking a word
                current_word_start = Some(index);
            }

            text.characters.push(Character {
                char,
                state: State::default(),
            });

            text
        });
    }

    /// Type input into the text.
    ///
    /// If `Some(char)` is given, the char will be added to the input and returned with it's [CharacterResult].
    /// If `None` is given, the text will delete a character from the input and returned with
    /// `CharacterResult::Deleted`.
    ///
    /// Returns `None` if trying to delete and empty input, or if the input is full (All text has
    /// been typed).
    pub fn input(&mut self, input: Option<char>) -> Option<(char, CharacterResult)> {
        if self.is_fully_typed() {
            return None;
        }

        input
            .and_then(|char| self.add_input(char).map(|result| (char, result)))
            .or_else(|| self.delete_input())
    }

    /// Updates the input and returns the typed characters new [State].
    ///
    /// Returns `None` if the input is full (All text has been typed).
    fn add_input(&mut self, input: char) -> Option<CharacterResult> {
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }

        let character = self.characters.get_mut(self.input.len())?;

        let result;
        let new_state;
        let prev_state = character.state;

        if character.char != input {
            new_state = State::Wrong;
            result = CharacterResult::Wrong;
        } else {
            result = match prev_state {
                State::None => {
                    new_state = State::Correct;
                    CharacterResult::Correct
                }
                State::WasWrong => {
                    new_state = State::Corrected;
                    CharacterResult::Corrected
                }
                State::WasCorrected => {
                    new_state = State::Corrected;
                    // This is not a mistake - The result of the input was that it was correctly
                    // typed because it was corrected before. But the state of the character should
                    // only be Corrected, as it once was Wrong.
                    CharacterResult::Correct
                }
                // The input was already typed - That shouldn't happen
                _ => unreachable!("Tried to add to already typed character!"),
            }
        }

        // Push input
        self.input.push(input);

        // Safety: We checked if started_at was none in the beginning of the function, and it wasn't.
        let started_at = self.started_at.as_ref().unwrap();

        // Update statistics with new length
        self.stats.update(
            input,
            result,
            self.input.len(),
            started_at.elapsed().as_secs_f64(),
        );

        // Update the character itself
        character.state = new_state;

        Some(result)
    }

    /// Deletes one char from the input and returns it, and it's previous [State]
    ///
    /// Returns `None` if the input is empty or full (All text has been typed)
    fn delete_input(&mut self) -> Option<(char, CharacterResult)> {
        // Delete the char from the input
        let deleted = self.input.pop()?;

        // Safety: No matter when the current function is called, because of the pop above
        // the input length should always be under or equal to the length of characters.
        let character = self
            .characters
            .get_mut(self.input.len())
            .expect("Failed to get current character");

        let prev_state = character.state;

        // Safety: We can't delete any input, unless input was already added.
        //         When input is added, we set the `started_at` property
        let started_at = self.started_at.as_ref().unwrap();

        // Update character
        match prev_state {
            State::Wrong => character.state = State::WasWrong,
            State::Corrected => character.state = State::WasCorrected,
            State::Correct => character.state = State::WasCorrect,
            // The input was not already typed - That shouldn't happen
            _ => unreachable!("Tried to delete a non-typed character!"),
        }

        let result = CharacterResult::Deleted(prev_state);

        // Update statistics
        self.stats.update(
            deleted,
            result,
            self.input.len(),
            started_at.elapsed().as_secs_f64(),
        );

        Some((deleted, result))
    }
}
