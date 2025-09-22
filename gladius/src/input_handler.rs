use crate::buffer::Buffer;
use crate::{CharacterResult, State};

/// Handles input processing and position tracking
#[derive(Debug, Clone)]
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
        text_buffer: &mut Buffer,
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
    fn add_input(&mut self, input: char, text_buffer: &mut Buffer) -> Option<CharacterResult> {
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
                State::WasCorrect => {
                    new_state = State::Correct;
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
    fn delete_input(&mut self, text_buffer: &mut Buffer) -> Option<(char, CharacterResult)> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::Buffer;

    #[test]
    fn test_input_handler_basic() {
        let mut text_buffer = Buffer::new("abc").unwrap();
        let mut input_handler = InputHandler::new();

        // Type correct character
        let result = input_handler
            .process_input(Some('a'), &mut text_buffer)
            .unwrap();
        assert_eq!(result.0, 'a');
        assert!(matches!(result.1, CharacterResult::Correct));
        assert_eq!(input_handler.input_len(), 1);
        assert!(!input_handler.is_input_empty());

        // Type wrong character
        let result = input_handler
            .process_input(Some('x'), &mut text_buffer)
            .unwrap();
        assert_eq!(result.0, 'x');
        assert!(matches!(result.1, CharacterResult::Wrong));
        assert_eq!(input_handler.input_len(), 2);

        // Delete 'x'
        let result = input_handler.process_input(None, &mut text_buffer).unwrap();
        assert_eq!(result.0, 'x');
        assert!(matches!(result.1, CharacterResult::Deleted(_)));
        assert_eq!(input_handler.input_len(), 1);

        // Type correct 'b'
        let result = input_handler
            .process_input(Some('b'), &mut text_buffer)
            .unwrap();
        assert_eq!(result.0, 'b');
        assert!(matches!(result.1, CharacterResult::Corrected));
        assert_eq!(input_handler.input_len(), 2);

        // Type correct 'c'
        let result = input_handler
            .process_input(Some('c'), &mut text_buffer)
            .unwrap();
        assert_eq!(result.0, 'c');
        assert!(matches!(result.1, CharacterResult::Correct));
        assert_eq!(input_handler.input_len(), 3);

        // Should be fully typed
        assert!(input_handler.is_fully_typed(text_buffer.text_len()));

        // Should return None when trying to input more
        assert!(
            input_handler
                .process_input(Some('d'), &mut text_buffer)
                .is_none()
        );
    }

    #[test]
    fn test_input_handler_deletion() {
        let mut text_buffer = Buffer::new("abc").unwrap();
        let mut input_handler = InputHandler::new();

        // Can't delete from empty input
        assert!(
            input_handler
                .process_input(None, &mut text_buffer)
                .is_none()
        );

        // Type a character then delete it
        input_handler
            .process_input(Some('a'), &mut text_buffer)
            .unwrap();
        assert_eq!(input_handler.input_len(), 1);

        let result = input_handler.process_input(None, &mut text_buffer).unwrap();
        assert_eq!(result.0, 'a');
        assert!(matches!(result.1, CharacterResult::Deleted(_)));
        assert_eq!(input_handler.input_len(), 0);
    }

    #[test]
    fn test_input_handler_correction_sequence() {
        let mut text_buffer = Buffer::new("abc").unwrap();
        let mut input_handler = InputHandler::new();

        // Type wrong, delete, type correct
        input_handler
            .process_input(Some('x'), &mut text_buffer)
            .unwrap(); // Wrong
        input_handler.process_input(None, &mut text_buffer).unwrap(); // Delete
        let result = input_handler
            .process_input(Some('a'), &mut text_buffer)
            .unwrap(); // Correct

        assert_eq!(result.0, 'a');
        assert!(matches!(result.1, CharacterResult::Corrected));
    }
}
