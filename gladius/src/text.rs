use web_time::Instant;

use crate::{Configuration, TempStatistics};

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
    /// TODO: MISSING IMPLEMENTATION
    pub state: State,
    pub string: String,
}

pub struct Character {
    pub char: char,
    pub state: State,
}

pub struct Text {
    characters: Vec<Character>,
    pub(crate) words: Vec<Word>,
    input: Vec<char>,
    stats: TempStatistics,
    config: Configuration,
    started_at: Option<Instant>,
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
        };

        text.push(string);

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

    /// Push more characters to the [Text].
    ///
    /// This allows for dynamically adding text during typing.
    pub fn push(&mut self, string: &str) {
        let mut current_word_start: Option<usize> = None;

        let chars: Vec<char> = string.chars().collect();

        let current_len = self.characters.len();
        let word_count = string.split_ascii_whitespace().count();
        let char_count = string.chars().count();
        self.characters.reserve(char_count);
        self.words.reserve(word_count);
        self.input.reserve(char_count);

        string
            .chars()
            .enumerate()
            .fold(&mut *self, |text, (index, char)| {
                // Find Word-indices and create Characters
                let is_whitespace = char.is_ascii_whitespace();

                if let Some(word_start) = current_word_start.take_if(|_| is_whitespace) {
                    // Add new word, as we've hit whitespace
                    text.words.push(Word {
                        start: word_start + current_len,
                        end: index + current_len,
                        state: State::default(),
                        string: chars[word_start..index].iter().collect(),
                    });
                } else if !is_whitespace && current_word_start.is_none() {
                    // Start tracking a word
                    current_word_start = Some(index);
                }

                text.characters.push(Character {
                    char,
                    state: State::default(),
                });

                text
            });

        if let Some(word_start) = current_word_start {
            self.words.push(Word {
                start: word_start + current_len,
                end: char_count + current_len,
                state: State::default(),
                // We know this is the last word, so we can just go unbounded
                string: chars[word_start..].iter().collect(),
            })
        }
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
            started_at.elapsed(),
            &self.config,
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
            started_at.elapsed(),
            &self.config,
        );

        Some((deleted, result))
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
        text.push(" world");
        assert_eq!(text.text_len(), 11);

        // Push empty string (should not change anything)
        text.push("");
        assert_eq!(text.text_len(), 11);

        // Push more text with special characters
        text.push("! 123");
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

        text.push(" second word");

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
        assert_eq!(text.words[0].end, 5);

        // Verify second word
        assert_eq!(text.words[1].string, "word");
        assert_eq!(text.words[1].start, 6);
        assert_eq!(text.words[1].end, 10);

        // Verify third word (from push)
        assert_eq!(text.words[2].string, "second");
        assert_eq!(text.words[2].start, 11);
        assert_eq!(text.words[2].end, 17);

        // Verify fourth word (from push)
        assert_eq!(text.words[3].string, "word");
        assert_eq!(text.words[3].start, 18);
        assert_eq!(text.words[3].end, 22);
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
        assert_eq!(stats.adds, 0);
        assert_eq!(stats.errors, 0);

        // Type wrong character
        text.input(Some('x')).unwrap();
        let stats = text.statistics();
        assert_eq!(stats.adds, 1);
        assert_eq!(stats.errors, 1);

        // Type correct character
        text.input(Some('b')).unwrap();
        let stats = text.statistics();
        assert_eq!(stats.adds, 2);
        assert_eq!(stats.errors, 1);
    }
}
