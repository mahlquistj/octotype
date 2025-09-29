//! # Gladius - High-Performance Typing Trainer Library
//!
//! Gladius is a comprehensive Rust library for building typing trainer applications.
//! It provides real-time typing analysis, flexible rendering systems, and detailed
//! performance statistics with a focus on accuracy, performance, and ease of use.
//!
//! ## Quick Start
//!
//! ```rust
//! use gladius::TypingSession;
//!
//! // Create a typing session
//! let mut session = TypingSession::new("Hello, world!").unwrap();
//!
//! // Process user input
//! while let Some((char, result)) = session.input(Some('H')) {
//!     println!("Typed '{}': {:?}", char, result);
//!     break; // Just for demo
//! }
//!
//! // Get progress and statistics
//! println!("Progress: {:.1}%", session.completion_percentage());
//! println!("WPM: {:.1}", session.statistics().measurements.last()
//!     .map(|m| m.wpm.raw).unwrap_or(0.0));
//! ```
//!
//! ## Key Features
//!
//! ### 🚀 **High Performance**
//! - **Fast character processing** - Amortized O(1) keystroke handling
//! - **O(1) word lookups** - Efficient character-to-word mapping
//! - **Optimized statistics** - Welford's algorithm for numerical stability
//! - **Memory efficient** - Minimal allocations during typing
//!
//! ### 📊 **Comprehensive Statistics**
//! - **Words per minute** (raw, corrected, actual)
//! - **Input per minute** (raw, actual)  
//! - **Accuracy percentages** (raw, actual)
//! - **Consistency analysis** with standard deviation
//! - **Detailed error tracking** by character and word
//! - **Real-time measurements** at configurable intervals
//!
//! ### 🎯 **Flexible Rendering**
//! - **Character-level rendering** with typing state information
//! - **Line-based rendering** with intelligent word wrapping
//! - **Cursor position tracking** across line boundaries
//! - **Unicode support** for international characters and emojis
//! - **Generic renderer interface** for any UI framework
//!
//! ### ⚙️ **Configurable Behavior**
//! - **Measurement intervals** for statistics collection
//! - **Line wrapping options** (word boundaries vs. character wrapping)
//! - **Newline handling** (respect or ignore paragraph breaks)
//! - **Performance tuning** for different use cases
//!
//! ## Architecture Overview
//!
//! Gladius is built with a modular architecture where each component has a specific responsibility:
//!
#![doc = simple_mermaid::mermaid!("../diagrams/architecture_overview.mmd")]
//!
//! ## Core Modules
//!
//! | Module | Purpose | Key Types |
//! |--------|---------|-----------|
//! | [`session`] | Session coordination and main API | [`TypingSession`] |
//! | [`buffer`] | Text storage and word/character management | [`Buffer`](buffer::Buffer) |
//! | [`input_handler`] | Keystroke processing and validation | [`InputHandler`](input_handler::InputHandler) |
//! | [`statistics`] | Performance data collection and analysis | [`Statistics`](statistics::Statistics), [`TempStatistics`](statistics::TempStatistics) |
//! | [`statistics_tracker`] | Real-time statistics coordination | [`StatisticsTracker`](statistics_tracker::StatisticsTracker) |
//! | [`render`] | Text display and line management | [`RenderingContext`](render::RenderingContext), [`LineContext`](render::LineContext) |
//! | [`math`] | Performance calculation algorithms | [`Wpm`](math::Wpm), [`Accuracy`](math::Accuracy), [`Consistency`](math::Consistency) |
//! | [`config`] | Runtime behavior configuration | [`Configuration`](config::Configuration) |
//!
//! ## Usage Examples
//!
//! ### Basic Typing Session
//!
//! ```rust
//! use gladius::TypingSession;
//! use gladius::CharacterResult;
//!
//! let mut session = TypingSession::new("The quick brown fox").unwrap();
//!
//! // Process typing input
//! match session.input(Some('T')) {
//!     Some((ch, CharacterResult::Correct)) => println!("Correct: {}", ch),
//!     Some((ch, CharacterResult::Wrong)) => println!("Wrong: {}", ch),
//!     Some((ch, CharacterResult::Corrected)) => println!("Corrected: {}", ch),
//!     Some((ch, CharacterResult::Deleted(state))) => println!("Deleted: {} (was {:?})", ch, state),
//!     None => println!("No input processed"),
//! }
//! ```
//!
//! ### Custom Configuration
//!
//! ```rust
//! use gladius::{TypingSession, config::Configuration};
//!
//! let config = Configuration {
//!     measurement_interval_seconds: 0.5, // More frequent measurements
//! };
//!
//! let session = TypingSession::new("Hello, world!")
//!     .unwrap()
//!     .with_configuration(config);
//! ```
//!
//! ### Character-level Rendering
//!
//! ```rust
//! use gladius::TypingSession;
//!
//! let session = TypingSession::new("hello").unwrap();
//!
//! let rendered: Vec<String> = session.render(|ctx| {
//!     let cursor = if ctx.has_cursor { " |" } else { "" };
//!     let state = match ctx.character.state {
//!         gladius::State::Correct => "✓",
//!         gladius::State::Wrong => "✗",
//!         gladius::State::None => "·",
//!         _ => "?",
//!     };
//!     format!("{}{}{}", ctx.character.char, state, cursor)
//! });
//! ```
//!
//! ### Line-based Rendering
//!
//! ```rust
//! use gladius::{TypingSession, render::LineRenderConfig};
//!
//! let session = TypingSession::new("The quick brown fox jumps over the lazy dog").unwrap();
//! let config = LineRenderConfig::new(20).with_word_wrapping(false);
//!
//! let lines: Vec<String> = session.render_lines(|line_ctx| {
//!     Some(line_ctx.contents.iter()
//!         .map(|ctx| ctx.character.char)
//!         .collect())
//! }, config);
//!
//! // Results in word-wrapped lines of ~20 characters each
//! ```
//!
//! ### Complete Session with Statistics
//!
//! ```rust
//! use gladius::{TypingSession, CharacterResult};
//!
//! let mut session = TypingSession::new("rust").unwrap();
//! let text_chars = ['r', 'u', 's', 't'];
//!
//! // Type the complete text
//! for ch in text_chars {
//!     session.input(Some(ch));
//! }
//!
//! // Get final statistics
//! if session.is_fully_typed() {
//!     let stats = session.finalize();
//!     println!("Final WPM: {:.1}", stats.wpm.raw);
//!     println!("Accuracy: {:.1}%", stats.accuracy.raw);
//!     println!("Total time: {:.2}s", stats.duration.as_secs_f64());
//!     println!("Character errors: {:?}", stats.counters.char_errors);
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! | Operation | Time Complexity | Notes |
//! |-----------|----------------|-------|
//! | Character input | O(1) amortized, O(w) worst case | Usually constant, worst case when recalculating word state |
//! | Character lookup | O(1) | Direct vector indexing |
//! | Word lookup | O(1) | Pre-computed mapping |
//! | Statistics update | O(1) typical, O(m) when measuring | Most updates are constant, measurements scan history |
//! | Rendering | O(n) | Linear in text length |
//! | Line wrapping | O(n) with O(w) lookahead | Linear with word boundary lookahead |
//! | Session creation | O(n) | One-time text parsing |
//!
//! ## Thread Safety
//!
//! Gladius types are not thread-safe by design for maximum performance. Each typing
//! session should be used on a single thread. Multiple sessions can run concurrently
//! on different threads.
//!
//! ## Memory Usage
//!
//! - **Text storage**: O(n) where n is text length
//! - **Statistics history**: O(k) where k is number of measurements
//! - **Input history**: O(m) where m is number of keystrokes
//! - **Word mapping**: O(n) pre-computed character-to-word index
//!
//! Memory usage is optimized for typing trainer use cases with efficient data structures
//! and minimal allocations during active typing.
//!
//! ## Minimum Supported Rust Version (MSRV)
//!
//! Gladius supports Rust 1.70.0 and later.

pub mod buffer;
pub mod config;
pub mod input_handler;
pub mod math;
pub mod render;
pub mod session;
pub mod statistics;
pub mod statistics_tracker;

/// Re-export of the main entry point for convenient access
pub use session::TypingSession;

// Shared types for readability and type safety
type Timestamp = f64;
type Minutes = f64;
type Float = f64;

/// Represents the current typing state of a character or word
///
/// States have a specific ordering that reflects their priority for word state calculations.
/// Higher priority states override lower priority ones when determining overall word state.
///
/// # State Transitions
///
/// ```text
/// None → Correct/Wrong → Deleted → Corrected (via new input)
/// ```
///
/// # Examples
///
/// ```rust
/// use gladius::State;
///
/// // Priority ordering (Higher states override lower ones)
/// assert!(State::Wrong > State::Corrected);
/// assert!(State::Corrected > State::Correct);
/// assert!(State::Correct > State::None);
/// ```
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

/// Result of processing a character input during typing
///
/// Indicates what happened when a character was typed or deleted, providing
/// detailed feedback about the typing action for statistics and UI updates.
///
/// # Ordering
///
/// Results are ordered by their impact on typing accuracy, with `Correct` being
/// the best outcome and `Deleted` potentially indicating typing inefficiency.
///
/// # Examples
///
/// ```rust
/// use gladius::{CharacterResult, State};
///
/// // Typing the correct character first time
/// let result = CharacterResult::Correct;
///
/// // Typing wrong, then deleting and typing correctly
/// let wrong = CharacterResult::Wrong;
/// let deleted = CharacterResult::Deleted(State::Wrong);
/// let corrected = CharacterResult::Corrected;
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CharacterResult {
    /// A character was deleted from the input (contains the previous state)
    Deleted(State),
    /// Character was typed incorrectly (doesn't match expected character)
    Wrong,
    /// Character was typed correctly after being previously wrong (correction)
    Corrected,
    /// Character was typed correctly on the first attempt
    Correct,
}

/// Represents a word in the text with its boundaries and typing state
///
/// Words are defined as sequences of non-whitespace characters separated by whitespace.
/// Each word tracks its position in the text and its overall typing state based on
/// the states of its constituent characters.
///
/// # Examples
///
/// ```rust
/// use gladius::{Word, State};
///
/// let word = Word {
///     start: 0,    // First character index
///     end: 4,      // Last character index + 1 (exclusive)
///     state: State::Correct,
/// };
///
/// // Check if a character index is part of this word
/// assert!(word.contains_index(&2));   // Character at index 2 is in the word
/// assert!(!word.contains_index(&5));  // Character at index 5 is not in the word
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Word {
    /// Starting character index (inclusive)
    pub start: usize,
    /// Ending character index (exclusive)
    pub end: usize,
    /// Current typing state of the word (highest priority state of any character)
    pub state: State,
}

impl Word {
    /// Check if a character index falls within this word's boundaries
    ///
    /// # Parameters
    ///
    /// * `index` - Character index to check
    ///
    /// # Returns
    ///
    /// `true` if the index is within [start, end), `false` otherwise
    pub fn contains_index(&self, index: &usize) -> bool {
        (self.start..self.end).contains(index)
    }
}

/// Represents a single character in the text with its typing state
///
/// Characters are the fundamental unit of typing analysis. Each character
/// maintains its Unicode value and current state based on user input.
///
/// # Examples
///
/// ```rust
/// use gladius::{Character, State};
///
/// let char = Character {
///     char: 'a',
///     state: State::Correct,
/// };
///
/// // Unicode characters are fully supported
/// let unicode_char = Character {
///     char: '🚀',
///     state: State::None,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Character {
    /// The Unicode character
    pub char: char,
    /// Current typing state of this character
    pub state: State,
}
