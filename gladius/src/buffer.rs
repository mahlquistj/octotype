//! # Buffer Module - Text Storage and Word/Character Management
//!
//! This module provides efficient text storage and parsing capabilities for typing trainers.
//! It manages the relationship between characters and words, tracks typing state, and provides
//! fast lookups for rendering and analysis.
//!
//! ## Key Features
//!
//! - **Efficient Text Parsing**: Breaks text into characters and words with proper boundaries
//! - **Fast Word Lookup**: O(1) character-to-word mapping for performance
//! - **State Tracking**: Maintains typing state for each character and word
//! - **Unicode Support**: Handles multi-byte characters correctly
//!
//! ## Data Structure
//!
#![doc = simple_mermaid::mermaid!("../diagrams/buffer_structure.mmd")]
//!
//! Data layout example: `"hello world"`
//! ```text
//! Characters: [h][e][l][l][o][ ][w][o][r][l][d]
//! Words:      [---word 0----]   [---word 1----]
//! Mapping:    [0][0][0][0][0][∅][1][1][1][1][1]
//! ```
//!
//! The buffer maintains three synchronized data structures:
//! - `characters`: Individual characters with their typing state
//! - `words`: Word boundaries and state information  
//! - `char_to_word_index`: Fast mapping from character to containing word

use crate::{Character, State, Word};

/// Text buffer with efficient character and word management
///
/// Stores parsed text as characters and words with fast lookup capabilities.
/// Designed for real-time typing applications where character state updates
/// and word boundary detection need to be performed efficiently.
///
/// # Performance Characteristics
///
/// - Character access: O(1)
/// - Word lookup by character: O(1)
/// - Text parsing: O(n) where n is text length
/// - State updates: O(1) per character
#[derive(Debug, Clone)]
pub struct Buffer {
    /// All characters in the text with their current typing state
    characters: Vec<Character>,
    /// Word boundaries and state information
    words: Vec<Word>,
    /// Maps each character index to its containing word (None for whitespace)
    char_to_word_index: Vec<Option<usize>>,
}

impl Buffer {
    /// Create a new buffer from text content
    ///
    /// Parses the input string into characters and words, building the internal
    /// data structures needed for efficient typing analysis.
    ///
    /// # Returns
    ///
    /// `None` if the input string is empty, otherwise a fully parsed `Buffer`.
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

    /// Get the total number of characters in the buffer
    pub fn text_len(&self) -> usize {
        self.characters.len()
    }

    /// Get a character by its index in the buffer
    pub fn get_character(&self, index: usize) -> Option<&Character> {
        self.characters.get(index)
    }

    /// Get the character that should be typed next
    ///
    /// Returns the character at the current input position, or the last
    /// character if the input has reached the end of the buffer.
    pub fn current_character(&self, input_len: usize) -> Option<&Character> {
        self.characters
            .get(input_len)
            .or_else(|| self.characters.last())
    }

    /// Find the word containing the character at the specified index
    ///
    /// Uses the internal character-to-word mapping for O(1) lookup performance.
    pub fn get_word_containing(&self, char_index: usize) -> Option<&Word> {
        let word_index = self.char_to_word_index.get(char_index).copied().flatten()?;
        self.words.get(word_index)
    }

    /// Find the word containing the character at the specified index (mutable)
    ///
    /// Uses the internal character-to-word mapping for O(1) lookup performance.
    pub fn get_word_containing_mut(&mut self, char_index: usize) -> Option<&mut Word> {
        let word_index = self.char_to_word_index.get(char_index).copied().flatten()?;
        self.words.get_mut(word_index)
    }

    /// Get a mutable reference to a character by its index
    pub fn get_character_mut(&mut self, index: usize) -> Option<&mut Character> {
        self.characters.get_mut(index)
    }

    /// Get all characters that belong to a specific word
    ///
    /// Returns a slice of characters from the word's start to end boundaries.
    pub fn get_word_characters(&self, word: &Word) -> &[Character] {
        &self.characters[word.start..word.end]
    }

    /// Get a word by its index in the word list
    pub fn get_word(&self, index: usize) -> Option<&Word> {
        self.words.get(index)
    }

    /// Get the total number of words in the buffer
    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    /// Get the word index for a character position (O(1) lookup)
    ///
    /// Returns the word index that contains the character at the given position.
    /// Returns None if the character is whitespace or the index is out of bounds.
    pub fn get_word_index_at(&self, char_index: usize) -> Option<usize> {
        self.char_to_word_index.get(char_index).copied().flatten()
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
        original_len: usize,
        current_word_start: &mut Option<usize>,
        current_word_index: &mut Option<usize>,
    ) {
        let is_whitespace = char.is_ascii_whitespace();

        if let Some(word_start) = current_word_start.take_if(|_| is_whitespace) {
            // Add new word, as we've hit whitespace
            self.add_word(word_start, index, original_len);
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
            self.char_to_word_index.push(Some(word_idx));
        } else {
            // Whitespace characters don't belong to any word
            self.char_to_word_index.push(None);
        }
    }

    /// Add a word to the words vector
    fn add_word(&mut self, word_start: usize, word_end: usize, original_len: usize) {
        self.words.push(Word {
            start: word_start + original_len,
            end: (word_end + original_len).saturating_sub(1),
            state: State::default(),
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
            });
        }
    }

    /// Add more text to the buffer
    ///
    /// Appends additional characters and words to the existing buffer,
    /// maintaining proper word boundaries and character-to-word mappings.
    /// Useful for dynamic text loading during typing sessions.
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
        let Some(word_index) = self.char_to_word_index.get(char_index).copied().flatten() else {
            // Skip whitespace characters (they map to usize::MAX)
            return;
        };

        let Some(word) = self.words.get_mut(word_index) else {
            return;
        };

        // If new character state is higher priority, upgrade word state immediately
        if new_character_state > word.state {
            word.state = new_character_state;
            return;
        }

        // If new character state is same or lower priority, check if recalculation is needed
        if new_character_state < word.state {
            // Only recalculate if the changed character might have been determining the word state
            // This happens when we're downgrading a character that was at the current word state level
            let word_start = word.start;
            let word_end = word.end;

            // Quick check: if any other character still has the current word state, no change needed
            let has_character_at_current_state = self.characters[word_start..word_end]
                .iter()
                .enumerate()
                .any(|(i, char)| word_start + i != char_index && char.state == word.state);

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_word_boundaries() {
        let mut text_buffer = Buffer::new("first word").unwrap();

        // Check initial words from "first word"
        assert_eq!(text_buffer.words.len(), 2);

        // Verify first word
        assert_eq!(text_buffer.words[0].start, 0);
        assert_eq!(text_buffer.words[0].end, 4);

        // Verify second word
        assert_eq!(text_buffer.words[1].start, 6);
        assert_eq!(text_buffer.words[1].end, 9);

        // Test push_string functionality
        text_buffer.push_string(" second word");

        // Verify text length after push
        assert_eq!(text_buffer.text_len(), 22);

        // Test that words are properly tracked with correct boundaries after push
        assert_eq!(text_buffer.words.len(), 4); // "first", "word", "second", "word"

        // Verify third word (from push)
        assert_eq!(text_buffer.words[2].start, 11);
        assert_eq!(text_buffer.words[2].end, 16);

        // Verify fourth word (from push)
        assert_eq!(text_buffer.words[3].start, 18);
        assert_eq!(text_buffer.words[3].end, 21);
    }
}
