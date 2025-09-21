use web_time::Instant;

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

pub struct Text {
    characters: Vec<Character>,
    words: Vec<Word>,
    input: Vec<char>,
    stats: TempStatistics,
    config: Configuration,
    started_at: Option<Instant>,
    /// Maps each character index to its corresponding word index for O(1) lookup
    char_to_word_index: Vec<usize>,
}

impl Text {
    pub fn new(string: &str) -> Option<Self> {
        if string.is_empty() {
            return None;
        }

        let mut text = Self {
            characters: vec![],
            words: vec![],
            input: vec![],
            stats: TempStatistics::default(),
            config: Configuration::default(),
            started_at: None,
            char_to_word_index: vec![],
        };

        text.push_string(string);

        Some(text)
    }

    /// Set configuration
    pub fn with_configuration(mut self, config: Configuration) -> Self {
        self.config = config;
        self
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

    /// Allocate capacity for the vectors based on expected size
    fn allocate_capacity(&mut self, char_count: usize, word_count: usize) {
        self.characters.reserve(char_count);
        self.words.reserve(word_count);
        self.input.reserve(char_count);
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

    /// Push more characters to the [Text].
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

        let index = self.input.len();
        let character = self.characters.get_mut(index)?;

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
            started_at.elapsed(),
            &self.config,
        );

        // Update the character itself
        character.state = new_state;

        self.update_word(index);

        Some(result)
    }

    /// Deletes one char from the input and returns it, and it's previous [State]
    ///
    /// Returns `None` if the input is empty or full (All text has been typed)
    fn delete_input(&mut self) -> Option<(char, CharacterResult)> {
        // Delete the char from the input
        let deleted = self.input.pop()?;

        let index = self.input.len();

        // Safety: No matter when the current function is called, because of the pop above
        // the input length should always be under or equal to the length of characters.
        let character = self
            .characters
            .get_mut(index)
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
            started_at.elapsed(),
            &self.config,
        );

        self.update_word(index);

        Some((deleted, result))
    }

    /// Update the word state to reflect it if it is correctly typed
    fn update_word(&mut self, at_index: usize) {
        // O(1) lookup using character-to-word index mapping
        let Some(&word_index) = self.char_to_word_index.get(at_index) else {
            return;
        };

        // Skip whitespace characters (they map to usize::MAX)
        if word_index == usize::MAX {
            return;
        }

        let Some(word) = self.words.get_mut(word_index) else {
            return;
        };

        let word_characters = &self.characters[word.start..word.end];

        // TODO: This could maybe be done more effeciently - Maybe pattern matching?
        let mut state = State::None;
        for character in word_characters {
            if character.state > state {
                state = character.state;
            }
        }

        word.state = state;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_new() {
        // Test with valid string
        let text = Text::new("hello world").unwrap();
        assert_eq!(text.text_len(), 11);
        assert_eq!(text.input_len(), 0);
        assert!(text.is_input_empty());
        assert!(!text.is_fully_typed());

        // Test with empty string
        let text = Text::new("");
        assert!(text.is_none());

        // Test with single character
        let text = Text::new("a").unwrap();
        assert_eq!(text.text_len(), 1);
        assert_eq!(text.current_character().char, 'a');

        // Test with unicode characters
        let text = Text::new("héllo wörld 🚀").unwrap();
        assert_eq!(text.text_len(), 13); // 13 Unicode code points
    }

    #[test]
    fn test_text_push() {
        let mut text = Text::new("hello").unwrap();
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
        let mut text = Text::new("first word").unwrap();

        // Debug: Check initial words from "first word"
        println!("Initial words: {:?}", text.words.len());
        for (i, word) in text.words.iter().enumerate() {
            println!(
                "Word {}: '{}' start={} end={}",
                i, word.string, word.start, word.end
            );
        }

        text.push_string(" second word");

        // Verify text length
        assert_eq!(text.text_len(), 22);

        // Debug: Check all words after push
        println!("After push words: {:?}", text.words.len());
        for (i, word) in text.words.iter().enumerate() {
            println!(
                "Word {}: '{}' start={} end={}",
                i, word.string, word.start, word.end
            );
        }

        // Test that words are properly tracked with correct boundaries
        assert_eq!(text.words.len(), 4); // "first", "word", "second", "word"

        // Verify first word
        assert_eq!(text.words[0].string, "first");
        assert_eq!(text.words[0].start, 0);
        assert_eq!(text.words[0].end, 4);

        // Verify second word
        assert_eq!(text.words[1].string, "word");
        assert_eq!(text.words[1].start, 6);
        assert_eq!(text.words[1].end, 9);

        // Verify third word (from push)
        assert_eq!(text.words[2].string, "second");
        assert_eq!(text.words[2].start, 11);
        assert_eq!(text.words[2].end, 16);

        // Verify fourth word (from push)
        assert_eq!(text.words[3].string, "word");
        assert_eq!(text.words[3].start, 18);
        assert_eq!(text.words[3].end, 21);
    }

    #[test]
    fn test_text_input_basic() {
        let mut text = Text::new("abc").unwrap();

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

        // Text should be fully typed
        assert!(text.is_fully_typed());

        // Should return None when trying to input more
        assert!(text.input(Some('d')).is_none());
    }

    #[test]
    fn test_text_deletion() {
        let mut text = Text::new("abc").unwrap();

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
        let mut text = Text::new("abc").unwrap();

        // Type wrong, delete, type correct
        text.input(Some('x')).unwrap(); // Wrong
        text.input(None).unwrap(); // Delete
        let result = text.input(Some('a')).unwrap(); // Correct

        assert_eq!(result.0, 'a');
        assert!(matches!(result.1, CharacterResult::Corrected));
    }

    #[test]
    fn test_text_unicode_support() {
        let mut text = Text::new("café 🚀").unwrap();
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
        let mut text = Text::new("ab").unwrap();

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
        let mut text = Text::new("hello world").unwrap();

        // Initially all words should have State::None
        assert_eq!(text.words[0].state, State::None); // "hello"
        assert_eq!(text.words[1].state, State::None); // "world"

        // Type first character correctly - word should become Correct
        text.input(Some('h')).unwrap();
        assert_eq!(text.words[0].state, State::Correct);
        assert_eq!(text.words[1].state, State::None);

        // Type second character correctly - word should remain Correct
        text.input(Some('e')).unwrap();
        assert_eq!(text.words[0].state, State::Correct);

        // Type third character wrong - word should become Wrong
        text.input(Some('x')).unwrap();
        assert_eq!(text.words[0].state, State::Wrong);

        // Delete the wrong character - word should become WasWrong
        text.input(None).unwrap();
        assert_eq!(text.words[0].state, State::WasWrong);

        // Type correct character - word should become Corrected
        text.input(Some('l')).unwrap();
        assert_eq!(text.words[0].state, State::Corrected);

        // Continue typing correctly - word should remain Corrected
        text.input(Some('l')).unwrap();
        text.input(Some('o')).unwrap();
        assert_eq!(text.words[0].state, State::Corrected);

        // Move to next word - type space correctly
        text.input(Some(' ')).unwrap();
        assert_eq!(text.words[0].state, State::Corrected);
        assert_eq!(text.words[1].state, State::None);

        // Type first character of second word correctly
        text.input(Some('w')).unwrap();
        assert_eq!(text.words[0].state, State::Corrected);
        assert_eq!(text.words[1].state, State::Correct);

        // Type wrong character in second word
        text.input(Some('x')).unwrap();
        assert_eq!(text.words[1].state, State::Wrong);

        // Delete and correct
        text.input(None).unwrap();
        assert_eq!(text.words[1].state, State::WasWrong);

        text.input(Some('o')).unwrap();
        assert_eq!(text.words[1].state, State::Corrected);

        // Type rest of second word correctly
        text.input(Some('r')).unwrap();
        text.input(Some('l')).unwrap();
        text.input(Some('d')).unwrap();
        assert_eq!(text.words[1].state, State::Corrected);

        // Test that a Corrected word becomes Wrong when typing a wrong character
        let mut text2 = Text::new("test").unwrap();

        // Create a corrected word by typing wrong, deleting, then correct
        text2.input(Some('x')).unwrap(); // Wrong
        text2.input(None).unwrap(); // Delete
        text2.input(Some('t')).unwrap(); // Correct (now Corrected)
        text2.input(Some('e')).unwrap(); // Correct
        assert_eq!(text2.words[0].state, State::Corrected);

        // Type wrong character - word should become Wrong (higher priority than Corrected)
        text2.input(Some('x')).unwrap();
        assert_eq!(text2.words[0].state, State::Wrong);
    }
}
