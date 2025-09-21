use crate::text::{Character, Word, State};

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

    /// Get word by index
    pub fn get_word(&self, index: usize) -> Option<&Word> {
        self.words.get(index)
    }

    /// Get number of words
    pub fn word_count(&self) -> usize {
        self.words.len()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_buffer_word_boundaries() {
        let mut text_buffer = TextBuffer::new("first word").unwrap();

        // Check initial words from "first word"
        assert_eq!(text_buffer.words.len(), 2);
        
        // Verify first word
        assert_eq!(text_buffer.words[0].string, "first");
        assert_eq!(text_buffer.words[0].start, 0);
        assert_eq!(text_buffer.words[0].end, 4);

        // Verify second word
        assert_eq!(text_buffer.words[1].string, "word");
        assert_eq!(text_buffer.words[1].start, 6);
        assert_eq!(text_buffer.words[1].end, 9);

        // Test push_string functionality
        text_buffer.push_string(" second word");

        // Verify text length after push
        assert_eq!(text_buffer.text_len(), 22);

        // Test that words are properly tracked with correct boundaries after push
        assert_eq!(text_buffer.words.len(), 4); // "first", "word", "second", "word"

        // Verify third word (from push)
        assert_eq!(text_buffer.words[2].string, "second");
        assert_eq!(text_buffer.words[2].start, 11);
        assert_eq!(text_buffer.words[2].end, 16);

        // Verify fourth word (from push)
        assert_eq!(text_buffer.words[3].string, "word");
        assert_eq!(text_buffer.words[3].start, 18);
        assert_eq!(text_buffer.words[3].end, 21);
    }
}