use crate::input_handler::InputHandler;
use crate::statistics_tracker::StatisticsTracker;
use crate::text_buffer::TextBuffer;
use crate::{Configuration, TempStatistics};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    // == Pre delete or add ==
    /// The text has never been touched
    #[default]
    None,

    // The below are in the specific order to support the `update_word` method properly

    // == Post add ==
    /// The text is correct
    Correct,
    /// The text was corrected
    Corrected,
    /// The text is wrong
    Wrong,

    // == Post delete ==
    /// The text was correct, but has since been deleted
    WasCorrect,
    /// The text was corrected, but has since been deleted
    WasCorrected,
    /// The text was wrong, but has since been deleted or corrected
    WasWrong,
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CharacterResult {
    Deleted(State),
    Wrong,
    Corrected,
    Correct,
}

pub struct RenderingContext<'a> {
    pub character: &'a Character,
    pub word: &'a Word,
    pub has_cursor: bool,
}

pub struct Word {
    pub start: usize,
    pub end: usize,
    pub string: String,
    pub state: State,
}

impl Word {
    pub fn contains_index(&self, index: &usize) -> bool {
        (self.start..self.end).contains(index)
    }
}

pub struct Character {
    pub char: char,
    pub state: State,
}

/// Main coordinator struct that provides the same API as the old Text struct
pub struct TypingSession {
    text_buffer: TextBuffer,
    input_handler: InputHandler,
    statistics: StatisticsTracker,
    config: Configuration,
}

impl TypingSession {
    pub fn new(string: &str) -> Option<Self> {
        let text_buffer = TextBuffer::new(string)?;

        Some(Self {
            text_buffer,
            input_handler: InputHandler::new(),
            statistics: StatisticsTracker::new(),
            config: Configuration::default(),
        })
    }

    /// Set configuration
    pub fn with_configuration(mut self, config: Configuration) -> Self {
        self.config = config;
        self
    }

    /// Returns the amount of characters currently in the text.
    pub fn text_len(&self) -> usize {
        self.text_buffer.text_len()
    }

    /// Returns the current character awaiting input.
    pub fn current_character(&self) -> &Character {
        // Safety: It's impossible for the user to create an empty TypingSession
        self.text_buffer
            .current_character(self.input_handler.input_len())
            .unwrap()
    }

    /// Returns true if the amount of characters currently in the input is 0.
    pub fn is_input_empty(&self) -> bool {
        self.input_handler.is_input_empty()
    }

    /// Returns the amount of characters currently in the input.
    pub fn input_len(&self) -> usize {
        self.input_handler.input_len()
    }

    /// Returns true if the text has been fully typed.
    pub fn is_fully_typed(&self) -> bool {
        self.input_handler
            .is_fully_typed(self.text_buffer.text_len())
    }

    /// Get the current statistics recorded for the text
    pub fn statistics(&self) -> &TempStatistics {
        self.statistics.statistics()
    }

    /// Push more characters to the text.
    pub fn push_string(&mut self, string: &str) {
        self.text_buffer.push_string(string);
    }

    /// Get word by index
    pub fn get_word(&self, index: usize) -> Option<&Word> {
        self.text_buffer.get_word(index)
    }

    /// Get number of words
    pub fn word_count(&self) -> usize {
        self.text_buffer.word_count()
    }

    /// Type input into the text.
    ///
    /// If `Some(char)` is given, the char will be added to the input and returned with it's [CharacterResult].
    /// If `None` is given, the text will delete a character from the input and returned with
    /// `CharacterResult::Deleted`.
    ///
    /// Returns `None` if trying to delete and the input is empty or full (All text has
    /// been typed).
    pub fn input(&mut self, input: Option<char>) -> Option<(char, CharacterResult)> {
        let result = self
            .input_handler
            .process_input(input, &mut self.text_buffer);

        // Update statistics if we got a result
        if let Some((char, char_result)) = result {
            self.statistics.update(
                char,
                char_result,
                self.input_handler.input_len(),
                &self.config,
            );
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_new() {
        // Test with valid string
        let text = TypingSession::new("hello world").unwrap();
        assert_eq!(text.text_len(), 11);
        assert_eq!(text.input_len(), 0);
        assert!(text.is_input_empty());
        assert!(!text.is_fully_typed());

        // Test with empty string
        let text = TypingSession::new("");
        assert!(text.is_none());

        // Test with single character
        let text = TypingSession::new("a").unwrap();
        assert_eq!(text.text_len(), 1);
        assert_eq!(text.current_character().char, 'a');

        // Test with unicode characters
        let text = TypingSession::new("héllo wörld 🚀").unwrap();
        assert_eq!(text.text_len(), 13); // 13 Unicode code points
    }

    #[test]
    fn test_text_push() {
        let mut text = TypingSession::new("hello").unwrap();
        assert_eq!(text.text_len(), 5);

        // Push additional text
        text.push_string(" world");
        assert_eq!(text.text_len(), 11);

        // Push empty string (should not change anything)
        text.push_string("");
        assert_eq!(text.text_len(), 11);

        // Push more text with special characters
        text.push_string("! 123");
        assert_eq!(text.text_len(), 16);

        // Test that we can still access current character
        assert_eq!(text.current_character().char, 'h');
    }

    #[test]
    fn test_text_unicode_support() {
        let mut text = TypingSession::new("café 🚀").unwrap();
        assert_eq!(text.text_len(), 6); // c, a, f, é, space, rocket emoji

        // Type unicode characters
        let result = text.input(Some('c')).unwrap();
        assert!(matches!(result.1, CharacterResult::Correct));

        let result = text.input(Some('a')).unwrap();
        assert!(matches!(result.1, CharacterResult::Correct));

        let result = text.input(Some('f')).unwrap();
        assert!(matches!(result.1, CharacterResult::Correct));

        let result = text.input(Some('é')).unwrap();
        assert!(matches!(result.1, CharacterResult::Correct));
    }

    #[test]
    fn test_update_word() {
        let mut text = TypingSession::new("hello world").unwrap();

        // Initially all words should have State::None
        assert_eq!(text.get_word(0).unwrap().state, State::None); // "hello"
        assert_eq!(text.get_word(1).unwrap().state, State::None); // "world"

        // Type first character correctly - word should become Correct
        text.input(Some('h')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Correct);
        assert_eq!(text.get_word(1).unwrap().state, State::None);

        // Type second character correctly - word should remain Correct
        text.input(Some('e')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Correct);

        // Type third character wrong - word should become Wrong
        text.input(Some('x')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Wrong);

        // Delete the wrong character - word should become WasWrong
        text.input(None).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::WasWrong);

        // Type correct character - word should become Corrected
        text.input(Some('l')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);

        // Continue typing correctly - word should remain Corrected
        text.input(Some('l')).unwrap();
        text.input(Some('o')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);

        // Move to next word - type space correctly
        text.input(Some(' ')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);
        assert_eq!(text.get_word(1).unwrap().state, State::None);

        // Type first character of second word correctly
        text.input(Some('w')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);
        assert_eq!(text.get_word(1).unwrap().state, State::Correct);

        // Type wrong character in second word
        text.input(Some('x')).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::Wrong);

        // Delete and correct
        text.input(None).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::WasWrong);

        text.input(Some('o')).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::Corrected);

        // Type rest of second word correctly
        text.input(Some('r')).unwrap();
        text.input(Some('l')).unwrap();
        text.input(Some('d')).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::Corrected);

        // Test that a Corrected word becomes Wrong when typing a wrong character
        let mut text2 = TypingSession::new("test").unwrap();

        // Create a corrected word by typing wrong, deleting, then correct
        text2.input(Some('x')).unwrap(); // Wrong
        text2.input(None).unwrap(); // Delete
        text2.input(Some('t')).unwrap(); // Correct (now Corrected)
        text2.input(Some('e')).unwrap(); // Correct
        assert_eq!(text2.get_word(0).unwrap().state, State::Corrected);

        // Type wrong character - word should become Wrong (higher priority than Corrected)
        text2.input(Some('x')).unwrap();
        assert_eq!(text2.get_word(0).unwrap().state, State::Wrong);
    }
}
