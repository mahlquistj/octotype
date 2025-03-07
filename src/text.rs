use std::{char, collections::HashSet};

#[derive(Debug)]
enum CharacterResult {
    Wrong(char), // TODO: Use character here to display multiple wrong characters after a word, like monkeytype does.
    Corrected, // TODO: Support seeing if a character was typed wrong before, but is now corrected.
    Right,
}

#[derive(Default, Debug)]
pub struct Segment {
    tokens: Vec<char>,
    input: Vec<CharacterResult>,
    wrong_inputs: HashSet<usize>,
}

impl Segment {
    pub fn is_done(&self) -> bool {
        self.input.len() == self.tokens.len()
    }

    pub fn add_input(&mut self, character: char) {
        let current = self.input.len();
        let matches_current = self.tokens[current] == character;
        let was_wrong = self.wrong_inputs.contains(&current);

        self.input.push(match (matches_current, was_wrong) {
            (true, false) => CharacterResult::Right,
            (true, true) => CharacterResult::Corrected,
            (false, _) => CharacterResult::Wrong(character),
        });
    }

    /// Deletes one char from the input. Returns false if input was empty.
    pub fn delete_input(&mut self) -> bool {
        self.input.pop().is_some()
    }

    pub fn length(&self) -> usize {
        self.tokens.len()
    }
}
