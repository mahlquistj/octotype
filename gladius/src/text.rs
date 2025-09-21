use web_time::{Duration, Instant};

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

/// Handles text storage, parsing, and word/character management
pub struct TextBuffer {
    characters: Vec<Character>,
    words: Vec<Word>,
    /// Maps each character index to its corresponding word index for O(1) lookup
    char_to_word_index: Vec<usize>,
}

impl TextBuffer {
    pub fn new(string: &str) -> Option<Self> {
        if string.is_empty() {
            return None;
        }

        let mut buffer = Self {
            characters: vec![],
            words: vec![],
            char_to_word_index: vec![],
        };

        buffer.push_string(string);
        Some(buffer)
    }

    /// Returns the amount of characters currently in the TextBuffer.
    pub fn text_len(&self) -> usize {
        self.characters.len()
    }

    /// Returns the character at the given index.
    pub fn get_character(&self, index: usize) -> Option<&Character> {
        self.characters.get(index)
    }

    /// Returns the current character awaiting input.
    pub fn current_character(&self, input_len: usize) -> Option<&Character> {
        self.characters
            .get(input_len)
            .or_else(|| self.characters.last())
    }

    /// Get the word that contains the character at the given index
    pub fn get_word_containing(&self, char_index: usize) -> Option<&Word> {
        let word_index = *self.char_to_word_index.get(char_index)?;
        if word_index == usize::MAX {
            return None; // Whitespace character
        }
        self.words.get(word_index)
    }

    /// Get mutable reference to the word that contains the character at the given index
    pub fn get_word_containing_mut(&mut self, char_index: usize) -> Option<&mut Word> {
        let word_index = *self.char_to_word_index.get(char_index)?;
        if word_index == usize::MAX {
            return None; // Whitespace character
        }
        self.words.get_mut(word_index)
    }

    /// Get character by index (mutable)
    pub fn get_character_mut(&mut self, index: usize) -> Option<&mut Character> {
        self.characters.get_mut(index)
    }

    /// Get slice of characters for a word
    pub fn get_word_characters(&self, word: &Word) -> &[Character] {
        &self.characters[word.start..word.end]
    }

    /// Allocate capacity for the vectors based on expected size
    fn allocate_capacity(&mut self, char_count: usize, word_count: usize) {
        self.characters.reserve(char_count);
        self.words.reserve(word_count);
        self.char_to_word_index.reserve(char_count);
    }

    /// Process each character and handle word boundary detection
    fn process_character(
        &mut self,
        char: char,
        index: usize,
        chars: &[char],
        original_len: usize,
        current_word_start: &mut Option<usize>,
        current_word_index: &mut Option<usize>,
    ) {
        let is_whitespace = char.is_ascii_whitespace();

        if let Some(word_start) = current_word_start.take_if(|_| is_whitespace) {
            // Add new word, as we've hit whitespace
            self.add_word(word_start, index, chars, original_len);
            *current_word_index = None;
        } else if !is_whitespace && current_word_start.is_none() {
            // Start tracking a word
            *current_word_start = Some(index);
            *current_word_index = Some(self.words.len()); // Next word index
        }

        // Add character
        self.characters.push(Character {
            char,
            state: State::default(),
        });

        // Map character to word index (or usize::MAX for whitespace)
        if let Some(word_idx) = *current_word_index {
            self.char_to_word_index.push(word_idx);
        } else {
            // Whitespace characters don't belong to any word
            self.char_to_word_index.push(usize::MAX);
        }
    }

    /// Add a word to the words vector
    fn add_word(
        &mut self,
        word_start: usize,
        word_end: usize,
        chars: &[char],
        original_len: usize,
    ) {
        self.words.push(Word {
            start: word_start + original_len,
            end: word_end + original_len - 1,
            state: State::default(),
            string: chars[word_start..word_end].iter().collect(),
        });
    }

    /// Handle the final word if the string doesn't end with whitespace
    fn finalize_last_word(
        &mut self,
        current_word_start: Option<usize>,
        chars: &[char],
        original_len: usize,
    ) {
        if let Some(word_start) = current_word_start {
            let char_count = chars.len();
            self.words.push(Word {
                start: word_start + original_len,
                end: char_count + original_len - 1,
                state: State::default(),
                string: chars[word_start..].iter().collect(),
            });
        }
    }

    /// Push more characters to the TextBuffer.
    ///
    /// This allows for dynamically adding text during typing.
    pub fn push_string(&mut self, string: &str) {
        let mut current_word_start: Option<usize> = None;
        let mut current_word_index: Option<usize> = None;

        let chars: Vec<char> = string.chars().collect();
        let word_count = string.split_ascii_whitespace().count();
        let char_count = chars.len();
        let original_len = self.characters.len();

        // Allocate capacity for efficient insertion
        self.allocate_capacity(char_count, word_count);

        // Process each character and build data structures directly
        for (index, char) in chars.iter().enumerate() {
            self.process_character(
                *char,
                index,
                &chars,
                original_len,
                &mut current_word_start,
                &mut current_word_index,
            );
        }

        // Handle the final word if string doesn't end with whitespace
        self.finalize_last_word(current_word_start, &chars, original_len);
    }

    /// Update word state incrementally based on a single character change
    pub fn update_word_state_incrementally(
        &mut self,
        char_index: usize,
        new_character_state: State,
    ) {
        let Some(&word_index) = self.char_to_word_index.get(char_index) else {
            return;
        };

        // Skip whitespace characters (they map to usize::MAX)
        if word_index == usize::MAX {
            return;
        }

        let Some(word) = self.words.get_mut(word_index) else {
            return;
        };

        let current_word_state = word.state;

        // If new character state is higher priority, upgrade word state immediately
        if new_character_state > current_word_state {
            word.state = new_character_state;
            return;
        }

        // If new character state is same or lower priority, check if recalculation is needed
        if new_character_state < current_word_state {
            // Only recalculate if the changed character might have been determining the word state
            // This happens when we're downgrading a character that was at the current word state level
            let word_start = word.start;
            let word_end = word.end;

            // Quick check: if any other character still has the current word state, no change needed
            let has_character_at_current_state = self.characters[word_start..word_end]
                .iter()
                .enumerate()
                .any(|(i, char)| word_start + i != char_index && char.state == current_word_state);

            if !has_character_at_current_state {
                // Need to recalculate word state from all characters
                self.recalculate_word_state(word_index);
            }
        }

        // If new_character_state == current_word_state, no change needed
    }

    /// Recalculate word state from all characters (fallback for edge cases)
    fn recalculate_word_state(&mut self, word_index: usize) {
        let Some(word) = self.words.get_mut(word_index) else {
            return;
        };

        let word_characters = &self.characters[word.start..word.end];
        let mut state = State::None;
        for character in word_characters {
            if character.state > state {
                state = character.state;
            }
        }
        word.state = state;
    }
}

/// Handles input processing and position tracking
pub struct InputHandler {
    input: Vec<char>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self { input: vec![] }
    }

    /// Returns true if the input is empty.
    pub fn is_input_empty(&self) -> bool {
        self.input.is_empty()
    }

    /// Returns the amount of characters currently in the input.
    pub fn input_len(&self) -> usize {
        self.input.len()
    }

    /// Returns true if the text has been fully typed.
    pub fn is_fully_typed(&self, text_len: usize) -> bool {
        self.input.len() == text_len
    }

    /// Process input (add or delete character)
    pub fn process_input(
        &mut self,
        input: Option<char>,
        text_buffer: &mut TextBuffer,
    ) -> Option<(char, CharacterResult)> {
        if self.is_fully_typed(text_buffer.text_len()) {
            return None;
        }

        input
            .and_then(|char| {
                self.add_input(char, text_buffer)
                    .map(|result| (char, result))
            })
            .or_else(|| self.delete_input(text_buffer))
    }

    /// Add character to input
    fn add_input(&mut self, input: char, text_buffer: &mut TextBuffer) -> Option<CharacterResult> {
        let index = self.input.len();
        let character = text_buffer.get_character_mut(index)?;

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

        // Update the character itself
        character.state = new_state;

        // Update word state
        text_buffer.update_word_state_incrementally(index, new_state);

        Some(result)
    }

    /// Delete character from input
    fn delete_input(&mut self, text_buffer: &mut TextBuffer) -> Option<(char, CharacterResult)> {
        // Delete the char from the input
        let deleted = self.input.pop()?;

        let index = self.input.len();

        // Safety: No matter when the current function is called, because of the pop above
        // the input length should always be under or equal to the length of characters.
        let character = text_buffer
            .get_character_mut(index)
            .expect("Failed to get current character");

        let prev_state = character.state;

        // Update character
        match prev_state {
            State::Wrong => character.state = State::WasWrong,
            State::Corrected => character.state = State::WasCorrected,
            State::Correct => character.state = State::WasCorrect,
            // The input was not already typed - That shouldn't happen
            _ => unreachable!("Tried to delete a non-typed character!"),
        }

        let result = CharacterResult::Deleted(prev_state);

        let character_state = character.state;
        // Update word state
        text_buffer.update_word_state_incrementally(index, character_state);

        Some((deleted, result))
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Handles statistics tracking and timing
pub struct StatisticsTracker {
    stats: TempStatistics,
    started_at: Option<Instant>,
}

impl StatisticsTracker {
    pub fn new() -> Self {
        Self {
            stats: TempStatistics::default(),
            started_at: None,
        }
    }

    /// Get the current statistics
    pub fn statistics(&self) -> &TempStatistics {
        &self.stats
    }

    /// Update statistics based on input result
    pub fn update(
        &mut self,
        char: char,
        result: CharacterResult,
        input_len: usize,
        config: &Configuration,
    ) {
        // Initialize timing on first input
        if self.started_at.is_none() {
            self.started_at = Some(Instant::now());
        }

        // Safety: We just set started_at above if it was None
        let started_at = self.started_at.as_ref().unwrap();
        let elapsed = started_at.elapsed();

        self.stats.update(char, result, input_len, elapsed, config);
    }

    /// Check if timing has started
    pub fn has_started(&self) -> bool {
        self.started_at.is_some()
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Option<Duration> {
        self.started_at.map(|start| start.elapsed())
    }
}

impl Default for StatisticsTracker {
    fn default() -> Self {
        Self::new()
    }
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

    /// Type input into the text.
    ///
    /// If `Some(char)` is given, the char will be added to the input and returned with it's [CharacterResult].
    /// If `None` is given, the text will delete a character from the input and returned with
    /// `CharacterResult::Deleted`.
    ///
    /// Returns `None` if trying to delete and empty input, or if the input is full (All text has
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
    fn test_text_word_boundaries() {
        let mut text = TypingSession::new("first word").unwrap();

        // Debug: Check initial words from "first word"
        println!("Initial words: {:?}", text.text_buffer.words.len());
        for (i, word) in text.text_buffer.words.iter().enumerate() {
            println!(
                "Word {}: '{}' start={} end={}",
                i, word.string, word.start, word.end
            );
        }

        text.push_string(" second word");

        let text_buffer = &text.text_buffer;

        // Verify text length
        assert_eq!(text.text_len(), 22);

        // Debug: Check all words after push
        println!("After push words: {:?}", text_buffer.words.len());
        for (i, word) in text_buffer.words.iter().enumerate() {
            println!(
                "Word {}: '{}' start={} end={}",
                i, word.string, word.start, word.end
            );
        }

        // Test that words are properly tracked with correct boundaries
        assert_eq!(text_buffer.words.len(), 4); // "first", "word", "second", "word"

        // Verify first word
        assert_eq!(text_buffer.words[0].string, "first");
        assert_eq!(text_buffer.words[0].start, 0);
        assert_eq!(text_buffer.words[0].end, 4);

        // Verify second word
        assert_eq!(text_buffer.words[1].string, "word");
        assert_eq!(text_buffer.words[1].start, 6);
        assert_eq!(text_buffer.words[1].end, 9);

        // Verify third word (from push)
        assert_eq!(text_buffer.words[2].string, "second");
        assert_eq!(text_buffer.words[2].start, 11);
        assert_eq!(text_buffer.words[2].end, 16);

        // Verify fourth word (from push)
        assert_eq!(text_buffer.words[3].string, "word");
        assert_eq!(text_buffer.words[3].start, 18);
        assert_eq!(text_buffer.words[3].end, 21);
    }

    #[test]
    fn test_text_input_basic() {
        let mut text = TypingSession::new("abc").unwrap();

        // Type correct character
        let result = text.input(Some('a')).unwrap();
        assert_eq!(result.0, 'a');
        assert!(matches!(result.1, CharacterResult::Correct));
        assert_eq!(text.input_len(), 1);
        assert!(!text.is_input_empty());

        // Type wrong character
        let result = text.input(Some('x')).unwrap();
        assert_eq!(result.0, 'x');
        assert!(matches!(result.1, CharacterResult::Wrong));
        assert_eq!(text.input_len(), 2);

        // Delete 'x'
        let result = text.input(None).unwrap();
        assert_eq!(result.0, 'x');
        assert!(matches!(result.1, CharacterResult::Deleted(_)));
        assert_eq!(text.input_len(), 1);

        // Type correct 'b'
        let result = text.input(Some('b')).unwrap();
        assert_eq!(result.0, 'b');
        assert!(matches!(result.1, CharacterResult::Corrected));
        assert_eq!(text.input_len(), 2);

        // Type correct 'c'
        let result = text.input(Some('c')).unwrap();
        assert_eq!(result.0, 'c');
        assert!(matches!(result.1, CharacterResult::Correct));
        assert_eq!(text.input_len(), 3);

        // TypingSession should be fully typed
        assert!(text.is_fully_typed());

        // Should return None when trying to input more
        assert!(text.input(Some('d')).is_none());
    }

    #[test]
    fn test_text_deletion() {
        let mut text = TypingSession::new("abc").unwrap();

        // Can't delete from empty input
        assert!(text.input(None).is_none());

        // Type a character then delete it
        text.input(Some('a')).unwrap();
        assert_eq!(text.input_len(), 1);

        let result = text.input(None).unwrap();
        assert_eq!(result.0, 'a');
        assert!(matches!(result.1, CharacterResult::Deleted(_)));
        assert_eq!(text.input_len(), 0);
    }

    #[test]
    fn test_text_correction_sequence() {
        let mut text = TypingSession::new("abc").unwrap();

        // Type wrong, delete, type correct
        text.input(Some('x')).unwrap(); // Wrong
        text.input(None).unwrap(); // Delete
        let result = text.input(Some('a')).unwrap(); // Correct

        assert_eq!(result.0, 'a');
        assert!(matches!(result.1, CharacterResult::Corrected));
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
    fn test_text_statistics_tracking() {
        let mut text = TypingSession::new("ab").unwrap();

        // Initially no statistics
        let stats = text.statistics();
        assert_eq!(stats.counters.adds, 0);
        assert_eq!(stats.counters.errors, 0);

        // Type wrong character
        text.input(Some('x')).unwrap();
        let stats = text.statistics();
        assert_eq!(stats.counters.adds, 1);
        assert_eq!(stats.counters.errors, 1);

        // Type correct character
        text.input(Some('b')).unwrap();
        let stats = text.statistics();
        assert_eq!(stats.counters.adds, 2);
        assert_eq!(stats.counters.errors, 1);
    }

    #[test]
    fn test_update_word() {
        let mut text = TypingSession::new("hello world").unwrap();

        // Initially all words should have State::None
        assert_eq!(text.text_buffer.words[0].state, State::None); // "hello"
        assert_eq!(text.text_buffer.words[1].state, State::None); // "world"

        // Type first character correctly - word should become Correct
        text.input(Some('h')).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::Correct);
        assert_eq!(text.text_buffer.words[1].state, State::None);

        // Type second character correctly - word should remain Correct
        text.input(Some('e')).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::Correct);

        // Type third character wrong - word should become Wrong
        text.input(Some('x')).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::Wrong);

        // Delete the wrong character - word should become WasWrong
        text.input(None).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::WasWrong);

        // Type correct character - word should become Corrected
        text.input(Some('l')).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::Corrected);

        // Continue typing correctly - word should remain Corrected
        text.input(Some('l')).unwrap();
        text.input(Some('o')).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::Corrected);

        // Move to next word - type space correctly
        text.input(Some(' ')).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::Corrected);
        assert_eq!(text.text_buffer.words[1].state, State::None);

        // Type first character of second word correctly
        text.input(Some('w')).unwrap();
        assert_eq!(text.text_buffer.words[0].state, State::Corrected);
        assert_eq!(text.text_buffer.words[1].state, State::Correct);

        // Type wrong character in second word
        text.input(Some('x')).unwrap();
        assert_eq!(text.text_buffer.words[1].state, State::Wrong);

        // Delete and correct
        text.input(None).unwrap();
        assert_eq!(text.text_buffer.words[1].state, State::WasWrong);

        text.input(Some('o')).unwrap();
        assert_eq!(text.text_buffer.words[1].state, State::Corrected);

        // Type rest of second word correctly
        text.input(Some('r')).unwrap();
        text.input(Some('l')).unwrap();
        text.input(Some('d')).unwrap();
        assert_eq!(text.text_buffer.words[1].state, State::Corrected);

        // Test that a Corrected word becomes Wrong when typing a wrong character
        let mut text2 = TypingSession::new("test").unwrap();

        // Create a corrected word by typing wrong, deleting, then correct
        text2.input(Some('x')).unwrap(); // Wrong
        text2.input(None).unwrap(); // Delete
        text2.input(Some('t')).unwrap(); // Correct (now Corrected)
        text2.input(Some('e')).unwrap(); // Correct
        assert_eq!(text2.text_buffer.words[0].state, State::Corrected);

        // Type wrong character - word should become Wrong (higher priority than Corrected)
        text2.input(Some('x')).unwrap();
        assert_eq!(text2.text_buffer.words[0].state, State::Wrong);
    }
}
