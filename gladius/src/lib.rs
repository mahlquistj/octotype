//! Gladius - A configurable library for creating typing trainers!

pub mod buffer;
pub mod config;
pub mod input_handler;
pub mod math;
pub mod render;
pub mod session;
pub mod statistics;
pub mod statistics_tracker;

pub use session::TypingSession;

/// The average word length in the english dictionary
const AVERAGE_WORD_LENGTH: usize = 5;

// Shared types for readability
type Timestamp = f64;
type Minutes = f64;
type Float = f64;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum State {
    // == Pre delete or add ==
    /// The text has never been touched
    #[default]
    None,

    // The below are in a specific order to updating words properly

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CharacterResult {
    Deleted(State),
    Wrong,
    Corrected,
    Correct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    pub start: usize,
    pub end: usize,
    pub state: State,
}

impl Word {
    pub fn contains_index(&self, index: &usize) -> bool {
        (self.start..self.end).contains(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Character {
    pub char: char,
    pub state: State,
}
