pub mod buffer;
pub mod config;
pub mod input_handler;
pub mod math;
pub mod render;
pub mod session;
pub mod statistics;
pub mod statistics_tracker;

pub use session::TypingSession;

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

pub struct Character {
    pub char: char,
    pub state: State,
}
