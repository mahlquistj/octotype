//! # Session Module - Complete Typing Session Management
//!
//! This module provides the high-level interface for managing complete typing sessions.
//! It coordinates all the core components (buffer, input handling, statistics, rendering)
//! to provide a unified API for typing trainer applications.
//!
//! ## Key Features
//!
//! - **Session Coordination**: Orchestrates text buffer, input processing, and statistics
//! - **Real-time Feedback**: Provides live statistics and progress tracking
//! - **Flexible Rendering**: Multiple rendering modes for different UI frameworks
//! - **Line Management**: Intelligent text wrapping and cursor tracking
//! - **Unicode Support**: Full support for international characters and emojis
//!
//! ## Session Lifecycle
//!
#![doc = simple_mermaid::mermaid!("../diagrams/session_lifecycle.mmd")]
//!
//! ## Usage Examples
//!
//! ### Basic Session
//!
//! ```rust
//! use gladius::session::TypingSession;
//! use gladius::CharacterResult;
//!
//! let mut session = TypingSession::new("hello world").unwrap();
//!
//! // Process typing input
//! let result = session.input(Some('h')).unwrap();
//! assert_eq!(result.0, 'h');
//! assert_eq!(result.1, CharacterResult::Correct);
//!
//! // Check progress
//! println!("Progress: {:.1}%", session.completion_percentage());
//! println!("Time elapsed: {:.2}s", session.time_elapsed());
//! ```
//!
//! ### Line-based Rendering
//!
//! ```rust
//! use gladius::session::TypingSession;
//! use gladius::render::LineRenderConfig;
//!
//! let session = TypingSession::new("hello world this is a test").unwrap();
//! let config = LineRenderConfig::new(10).with_word_wrapping(false);
//!
//! let lines: Vec<String> = session.render_lines(|line_context| {
//!     Some(line_context.contents.iter()
//!         .map(|ctx| ctx.character.char)
//!         .collect())
//! }, config);
//!
//! // Results in ["hello", "world this", "is a test"]
//! ```

use crate::buffer::Buffer;
use crate::config::Configuration;
use crate::input_handler::InputHandler;
use crate::render::{LineContext, LineRenderConfig, RenderingContext, RenderingIterator};
use crate::statistics::{Statistics, TempStatistics};
use crate::statistics_tracker::StatisticsTracker;
use crate::{Character, CharacterResult, Word};
use web_time::Duration;

/// Complete typing session coordinator and state manager
///
/// Represents a single typing practice session with integrated text management,
/// input processing, statistics tracking, and rendering capabilities. This is the
/// main entry point for typing trainer applications.
///
/// # Architecture
///
/// The TypingSession coordinates four main components:
/// - **Buffer**: Text storage and word/character management
/// - **InputHandler**: Keystroke processing and validation
/// - **StatisticsTracker**: Real-time performance data collection
/// - **Configuration**: Runtime behavior settings
///
/// # Performance
///
/// - Character input processing: O(1) per keystroke
/// - Rendering: O(n) where n is text length
/// - Line rendering: O(n) with intelligent word wrapping
/// - Memory usage: O(n) for text storage plus O(k) for statistics history
///
/// # Thread Safety
///
/// TypingSession is not thread-safe. Each session should be used on a single thread.
/// Multiple sessions can run concurrently on different threads.
///
/// # Examples
///
/// ```rust
/// use gladius::session::TypingSession;
/// use gladius::config::Configuration;
///
/// // Create a basic session
/// let mut session = TypingSession::new("Hello, world!").unwrap();
///
/// // Process typing with error handling
/// while !session.is_fully_typed() {
///     // In a real app, get input from user
///     if let Some(result) = session.input(Some('H')) {
///         println!("Typed '{}': {:?}", result.0, result.1);
///     }
///     break; // Just demo - don't actually loop infinitely
/// }
///
/// // Get final statistics when complete
/// if session.is_fully_typed() {
///     let stats = session.finalize();
///     println!("WPM: {:.1}", stats.wpm.raw);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TypingSession {
    /// Text buffer containing characters, words, and typing state
    text_buffer: Buffer,
    /// Input processor for keystroke validation and state management
    input_handler: InputHandler,
    /// Statistics collector for performance tracking
    statistics: StatisticsTracker,
    /// Configuration for measurement intervals and behavior
    config: Configuration,
}

impl TypingSession {
    /// Create a new typing session with the given text
    ///
    /// Initializes all components with default settings and prepares the session
    /// for input processing. The text is parsed into characters and words for
    /// efficient access during typing.
    ///
    /// # Parameters
    ///
    /// * `string` - The text to be typed (must be non-empty)
    ///
    /// # Returns
    ///
    /// `Some(TypingSession)` if the text is valid, `None` if empty
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::session::TypingSession;
    ///
    /// // Create session with simple text
    /// let session = TypingSession::new("Hello, world!").unwrap();
    /// assert_eq!(session.text_len(), 13);
    ///
    /// // Unicode support
    /// let session = TypingSession::new("café 🚀").unwrap();
    /// assert_eq!(session.text_len(), 6);
    ///
    /// // Empty text returns None
    /// assert!(TypingSession::new("").is_none());
    /// ```
    pub fn new(string: &str) -> Option<Self> {
        let text_buffer = Buffer::new(string)?;

        Some(Self {
            text_buffer,
            input_handler: InputHandler::new(),
            statistics: StatisticsTracker::new(),
            config: Configuration::default(),
        })
    }

    /// Configure the session with custom settings (builder pattern)
    ///
    /// # Parameters
    ///
    /// * `config` - Configuration for measurement intervals and behavior
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::session::TypingSession;
    /// use gladius::config::Configuration;
    ///
    /// let config = Configuration {
    ///     measurement_interval_seconds: 0.5, // More frequent measurements
    /// };
    ///
    /// let session = TypingSession::new("hello world")
    ///     .unwrap()
    ///     .with_configuration(config);
    /// ```
    pub fn with_configuration(mut self, config: Configuration) -> Self {
        self.config = config;
        self
    }

    /// Get a character by its index in the text
    ///
    /// Returns the character data including its current typing state.
    /// Useful for custom rendering and analysis.
    ///
    /// # Parameters
    ///
    /// * `index` - Zero-based character index
    ///
    /// # Returns
    ///
    /// `Some(&Character)` if index is valid, `None` otherwise
    pub fn get_character(&self, index: usize) -> Option<&Character> {
        self.text_buffer.get_character(index)
    }

    /// Get word containing index
    pub fn get_word_containing_index(&self, index: usize) -> Option<&Word> {
        self.text_buffer.get_word_containing(index)
    }

    /// Get the total number of characters in the text
    ///
    /// Returns the length of the complete text including spaces and punctuation.
    /// This represents the target length that the user needs to type.
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

    /// Get the number of characters currently typed
    ///
    /// Returns the current position in the text, representing how many
    /// characters the user has typed so far (including errors).
    pub fn input_len(&self) -> usize {
        self.input_handler.input_len()
    }

    /// Check if the entire text has been successfully typed
    ///
    /// Returns true when the user has typed all characters in the text.
    /// At this point, the session can be finalized to get complete statistics.
    pub fn is_fully_typed(&self) -> bool {
        self.input_handler
            .is_fully_typed(self.text_buffer.text_len())
    }

    /// Get the typing completion percentage
    ///
    /// Returns a value between 0.0 and 100.0 representing how much of the
    /// text has been typed so far.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::session::TypingSession;
    ///
    /// let mut session = TypingSession::new("hello").unwrap();
    /// assert_eq!(session.completion_percentage(), 0.0);
    ///
    /// session.input(Some('h')).unwrap(); // 1/5 = 20%
    /// assert_eq!(session.completion_percentage(), 20.0);
    /// ```
    pub fn completion_percentage(&self) -> f64 {
        let input_len = self.input_handler.input_len();
        let text_len = self.text_buffer.text_len();

        if text_len == 0 {
            return 0.0;
        }

        (input_len as f64 / text_len as f64) * 100.0
    }

    /// Get the elapsed time since the session started
    ///
    /// Returns the time in seconds from the first keystroke to now.
    /// Returns 0.0 if no input has been processed yet.
    pub fn time_elapsed(&self) -> f64 {
        self.statistics
            .total_duration()
            .as_ref()
            .map(Duration::as_secs_f64)
            .unwrap_or(0.0)
    }

    /// Get real-time statistics for the current session
    ///
    /// Returns live statistics including measurements, counters, and input history.
    /// Use this for displaying real-time performance feedback during typing.
    pub fn statistics(&self) -> &TempStatistics {
        self.statistics.statistics()
    }

    /// Push more characters to the text.
    pub fn push_string(&mut self, string: &str) {
        self.text_buffer.push_string(string);
    }

    /// Get word by index
    pub fn get_word(&self, index: usize) -> Option<&Word> {
        self.text_buffer.get_word(index)
    }

    /// Get number of words
    pub fn word_count(&self) -> usize {
        self.text_buffer.word_count()
    }

    /// Get the number of words the user has completely typed
    ///
    /// Returns the count of words that have been fully typed by the user.
    /// Iterates through words to find the last completed one.
    ///
    /// # Performance
    ///
    /// - Time complexity: O(w) where w is the number of words in the text
    /// - Space complexity: O(1)
    /// - Average case: O(completed_words) due to early break when finding incomplete word
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::session::TypingSession;
    ///
    /// let mut session = TypingSession::new("hello world test").unwrap();
    /// assert_eq!(session.words_typed_count(), 0);
    ///
    /// // Type "hello "
    /// for ch in "hello ".chars() {
    ///     session.input(Some(ch));
    /// }
    /// assert_eq!(session.words_typed_count(), 1); // "hello" is complete
    ///
    /// // Type "wo"
    /// session.input(Some('w'));
    /// session.input(Some('o'));
    /// assert_eq!(session.words_typed_count(), 1); // "world" still incomplete
    /// ```
    pub fn words_typed_count(&self) -> usize {
        let input_len = self.input_len();

        // No input means no words typed
        if input_len == 0 {
            return 0;
        }

        // Find the highest word index that has been completely typed
        // A word is completely typed when we've typed past its end boundary
        let mut completed_words = 0;

        for word_index in 0..self.text_buffer.word_count() {
            if let Some(word) = self.text_buffer.get_word(word_index) {
                // Account for the off-by-one in word boundaries - add 1 to end
                // The actual word includes one more character than the stored end
                if input_len > word.end {
                    // We've typed past the end of this word (including any following space)
                    completed_words = word_index + 1;
                } else {
                    // We haven't completed this word yet, so we're done
                    break;
                }
            }
        }

        completed_words
    }

    /// Render the text using a generic renderer function
    pub fn render<Char, F: FnMut(RenderingContext) -> Char>(&self, mut renderer: F) -> Vec<Char> {
        let mut results = Vec::with_capacity(self.text_len());
        let cursor_position = self.input_len();

        for i in 0..self.text_len() {
            let character = self.text_buffer.get_character(i).unwrap();
            let word = self.text_buffer.get_word_containing(i);
            let has_cursor = i == cursor_position;

            let context = RenderingContext {
                character,
                word,
                has_cursor,
                index: i,
            };

            results.push(renderer(context));
        }

        results
    }

    /// Render the text as lines with word wrapping and line management
    ///
    /// Breaks the text into lines according to the configuration and applies
    /// the provided renderer function to each line.
    ///
    /// # Performance
    ///
    /// - Time complexity: O(n) where n is text length, with O(w) lookahead for word wrapping
    /// - Space complexity: O(n) for storing line contexts
    /// - Word wrapping adds constant factor overhead for lookahead scanning
    pub fn render_lines<Line, F: FnMut(LineContext) -> Option<Line>>(
        &self,
        mut line_renderer: F,
        config: LineRenderConfig,
    ) -> Vec<Line> {
        let mut lines = Vec::new();
        let mut current_line_contexts = Vec::new();
        let mut current_line_length = 0;
        let mut cursor_line_index = None;

        for context in self.render_iter() {
            let char_is_space = context.character.char.is_ascii_whitespace();
            let char_is_newline = context.character.char == '\n';
            let context_index = context.index;
            let has_cursor = context.has_cursor;

            // Track which line the cursor is on
            if has_cursor {
                cursor_line_index = Some(lines.len()); // Current line being built
            }

            // Handle newline breaking if enabled
            if config.break_at_newlines && char_is_newline {
                // Add the newline context to the current line, then break
                current_line_contexts.push(context);
                lines.push((current_line_contexts, lines.len()));
                current_line_contexts = Vec::new();
                current_line_length = 0;
                continue;
            }

            // If we're at a space and not wrapping words, consider breaking here
            // if we're approaching the line limit
            if !config.wrap_words && char_is_space && current_line_length > 0 {
                // Look ahead to see if the next word would fit
                let mut look_ahead_length = 0;
                let mut look_ahead_index = context_index + 1;

                // Count characters until next space or end
                while look_ahead_index < self.text_len() {
                    if let Some(look_ahead_char) = self.get_character(look_ahead_index) {
                        if look_ahead_char.char.is_ascii_whitespace() {
                            break;
                        }
                        look_ahead_length += 1;
                        look_ahead_index += 1;
                    } else {
                        break;
                    }
                }

                // If adding the the next word and the space after it would exceed the line length,
                // add the space to current line then break
                if current_line_length + 1 + look_ahead_length > config.line_length {
                    // Add the space to the current line first
                    current_line_contexts.push(context);
                    // Then break the line
                    lines.push((current_line_contexts, lines.len())); // Store line with its index
                    current_line_contexts = Vec::new();
                    current_line_length = 0;
                    continue; // Continue to next iteration
                }
            }

            // Check if adding this character would exceed line length
            if current_line_length >= config.line_length {
                // We need to wrap
                lines.push((current_line_contexts, lines.len())); // Store line with its index
                current_line_contexts = Vec::new();
                current_line_length = 0;

                // Skip whitespace at the beginning of new lines
                if char_is_space {
                    continue;
                }
            }

            current_line_contexts.push(context);
            current_line_length += 1;
        }

        // Add the final line if it has content
        if !current_line_contexts.is_empty() {
            lines.push((current_line_contexts, lines.len()));
        }

        // If cursor is at the end of text, it's on the last line
        if cursor_line_index.is_none() {
            cursor_line_index = Some(lines.len().saturating_sub(1));
        }

        // Convert to final result with proper line offsets
        let cursor_line = cursor_line_index.unwrap_or(0);
        lines
            .into_iter()
            .filter_map(|(line_contexts, line_index)| {
                let line_context = LineContext {
                    active_line_offset: line_index as isize - cursor_line as isize,
                    contents: line_contexts,
                };
                line_renderer(line_context)
            })
            .collect()
    }

    /// Create an iterator over rendering contexts
    pub fn render_iter(&self) -> RenderingIterator<'_> {
        self.into()
    }

    /// Process a typing input and update the session state
    ///
    /// This is the main method for handling user input during typing. It processes
    /// character input or deletions, updates statistics, validates correctness,
    /// and automatically handles session completion.
    ///
    /// # Parameters
    ///
    /// * `input` - `Some(char)` to type a character, `None` to delete the last character
    ///
    /// # Returns
    ///
    /// * `Some((char, result))` - The character and its validation result
    /// * `None` - If no input could be processed (empty input on deletion, or session complete)
    ///
    /// # Character Results
    ///
    /// - `Correct`: Character matches expected and was typed correctly
    /// - `Wrong`: Character doesn't match expected character
    /// - `Corrected`: Character matches expected but was previously typed incorrectly
    /// - `Deleted(state)`: Character was deleted, with its previous state
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::session::TypingSession;
    /// use gladius::CharacterResult;
    ///
    /// let mut session = TypingSession::new("hello").unwrap();
    ///
    /// // Type correct character
    /// let result = session.input(Some('h')).unwrap();
    /// assert_eq!(result.0, 'h');
    /// assert_eq!(result.1, CharacterResult::Correct);
    ///
    /// // Type wrong character  
    /// let result = session.input(Some('x')).unwrap();
    /// assert_eq!(result.0, 'x');
    /// assert_eq!(result.1, CharacterResult::Wrong);
    ///
    /// // Delete wrong character
    /// let result = session.input(None).unwrap();
    /// assert_eq!(result.0, 'x');
    /// assert!(matches!(result.1, CharacterResult::Deleted(_)));
    ///
    /// // Type correct character (now corrected)
    /// let result = session.input(Some('e')).unwrap();
    /// assert_eq!(result.0, 'e');
    /// assert_eq!(result.1, CharacterResult::Corrected);
    /// ```
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

            // Check if typing is now complete and mark completion
            if self.is_fully_typed() && !self.statistics.is_completed() {
                self.statistics.mark_completed();
            }
        }

        result
    }

    /// Finalize the session and generate complete statistics
    ///
    /// Consumes the session and returns comprehensive final statistics including
    /// all performance metrics, measurements, and detailed analysis. This should
    /// only be called when the session is complete.
    ///
    /// # Returns
    ///
    /// * `Ok(Statistics)` - Complete session statistics
    /// * `Err(message)` - If the session is not yet complete
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::session::TypingSession;
    ///
    /// let mut session = TypingSession::new("hi").unwrap();
    ///
    /// // Type the complete text
    /// session.input(Some('h')).unwrap();
    /// session.input(Some('i')).unwrap();
    ///
    /// // Now we're done
    /// let stats = session.finalize();
    /// assert_eq!(stats.counters.corrects, 2);
    /// assert_eq!(stats.counters.errors, 0);
    /// ```
    pub fn finalize(self) -> Statistics {
        let text_len = self.text_len();
        self.statistics.finalize(text_len)
    }
}

#[cfg(test)]
mod tests {
    use crate::State;

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
    fn test_update_word() {
        let mut text = TypingSession::new("hello world").unwrap();

        // Initially all words should have State::None
        assert_eq!(text.get_word(0).unwrap().state, State::None); // "hello"
        assert_eq!(text.get_word(1).unwrap().state, State::None); // "world"

        // Type first character correctly - word should become Correct
        text.input(Some('h')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Correct);
        assert_eq!(text.get_word(1).unwrap().state, State::None);

        // Type second character correctly - word should remain Correct
        text.input(Some('e')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Correct);

        // Type third character wrong - word should become Wrong
        text.input(Some('x')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Wrong);

        // Delete the wrong character - word should become WasWrong
        text.input(None).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::WasWrong);

        // Type correct character - word should become Corrected
        text.input(Some('l')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);

        // Continue typing correctly - word should remain Corrected
        text.input(Some('l')).unwrap();
        text.input(Some('o')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);

        // Move to next word - type space correctly
        text.input(Some(' ')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);
        assert_eq!(text.get_word(1).unwrap().state, State::None);

        // Type first character of second word correctly
        text.input(Some('w')).unwrap();
        assert_eq!(text.get_word(0).unwrap().state, State::Corrected);
        assert_eq!(text.get_word(1).unwrap().state, State::Correct);

        // Type wrong character in second word
        text.input(Some('x')).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::Wrong);

        // Delete and correct
        text.input(None).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::WasWrong);

        text.input(Some('o')).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::Corrected);

        // Type rest of second word correctly
        text.input(Some('r')).unwrap();
        text.input(Some('l')).unwrap();
        text.input(Some('d')).unwrap();
        assert_eq!(text.get_word(1).unwrap().state, State::Corrected);

        // Test that a Corrected word becomes Wrong when typing a wrong character
        let mut text2 = TypingSession::new("test").unwrap();

        // Create a corrected word by typing wrong, deleting, then correct
        text2.input(Some('x')).unwrap(); // Wrong
        text2.input(None).unwrap(); // Delete
        text2.input(Some('t')).unwrap(); // Correct (now Corrected)
        text2.input(Some('e')).unwrap(); // Correct
        assert_eq!(text2.get_word(0).unwrap().state, State::Corrected);

        // Type wrong character - word should become Wrong (higher priority than Corrected)
        text2.input(Some('x')).unwrap();
        assert_eq!(text2.get_word(0).unwrap().state, State::Wrong);
    }

    #[test]
    fn test_rendering() {
        let mut text = TypingSession::new("hello").unwrap();

        // Type some characters
        text.input(Some('h')).unwrap(); // Correct
        text.input(Some('x')).unwrap(); // Wrong

        // Test render method
        let rendered: Vec<String> = text.render(|ctx| {
            let state_str = match ctx.character.state {
                State::None => "none",
                State::Correct => "correct",
                State::Wrong => "wrong",
                _ => "other",
            };
            let cursor_str = if ctx.has_cursor { " [cursor]" } else { "" };
            format!("{}:{}{}", ctx.character.char, state_str, cursor_str)
        });

        assert_eq!(rendered.len(), 5);
        assert_eq!(rendered[0], "h:correct");
        assert_eq!(rendered[1], "e:wrong");
        assert_eq!(rendered[2], "l:none [cursor]");
        assert_eq!(rendered[3], "l:none");
        assert_eq!(rendered[4], "o:none");

        // Test render_iter method
        let rendered_iter: Vec<char> = text.render_iter().map(|ctx| ctx.character.char).collect();

        assert_eq!(rendered_iter, vec!['h', 'e', 'l', 'l', 'o']);

        // Test that iterator has correct size
        let iter = text.render_iter();
        assert_eq!(iter.len(), 5);
        assert_eq!(iter.size_hint(), (5, Some(5)));
    }

    #[test]
    fn test_completion_and_finalization() {
        let mut text = TypingSession::new("hi").unwrap();

        // Initially not completed
        assert!(!text.is_fully_typed());

        // Type first character
        text.input(Some('h')).unwrap();
        assert!(!text.is_fully_typed());

        // Type second character - should complete the session
        text.input(Some('i')).unwrap();
        assert!(text.is_fully_typed());

        // Finalize
        let stats = text.finalize();

        // Verify the statistics contain expected data
        assert_eq!(stats.counters.adds, 2);
        assert_eq!(stats.counters.corrects, 2);
        assert_eq!(stats.counters.errors, 0);
    }

    #[test]
    fn test_finalization_before_completion() {
        let text = TypingSession::new("hello").unwrap();

        // Try to finalize without completing
        text.finalize();
    }

    #[test]
    fn test_render_lines() {
        let text = TypingSession::new("hello world this is a test").unwrap();

        // Test with word wrapping disabled
        let lines: Vec<String> = text.render_lines(
            |line_ctx| {
                Some(
                    line_ctx
                        .contents
                        .iter()
                        .map(|ctx| ctx.character.char)
                        .collect::<String>(),
                )
            },
            LineRenderConfig::new(10).with_word_wrapping(false), // config
        );

        // Should break at word boundaries
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "hello ");
        assert_eq!(lines[1], "world this ");
        assert_eq!(lines[2], "is a test");

        // Test with word wrapping enabled
        let lines_wrapped: Vec<String> = text.render_lines(
            |line_ctx| {
                Some(
                    line_ctx
                        .contents
                        .iter()
                        .map(|ctx| ctx.character.char)
                        .collect::<String>(),
                )
            },
            LineRenderConfig::new(10).with_word_wrapping(true), // config
        );

        // Should break at exactly 10 characters
        assert_eq!(lines_wrapped.len(), 3);
        assert_eq!(lines_wrapped[0], "hello worl");
        assert_eq!(lines_wrapped[1], "d this is ");
        assert_eq!(lines_wrapped[2], "a test");
    }

    #[test]
    fn test_render_lines_with_line_context() {
        let text = TypingSession::new("one two three").unwrap();

        let lines: Vec<(isize, String)> = text.render_lines(
            |line_ctx| {
                Some((
                    line_ctx.active_line_offset,
                    line_ctx
                        .contents
                        .iter()
                        .map(|ctx| ctx.character.char)
                        .collect::<String>(),
                ))
            },
            LineRenderConfig::new(5).with_word_wrapping(false), // config
        );

        assert_eq!(lines.len(), 3);
        // Cursor is at position 0, which is in the first line (line 0)
        // So line 0 has offset 0, line 1 has offset 1, line 2 has offset 2
        assert_eq!(lines[0], (0, "one ".to_string())); // cursor line - offset 0
        assert_eq!(lines[1], (1, "two ".to_string())); // 1 line after cursor
        assert_eq!(lines[2], (2, "three".to_string())); // 2 lines after cursor
    }

    #[test]
    fn test_render_lines_cursor_in_middle() {
        let mut text = TypingSession::new("one two three four").unwrap();

        // Type some characters to move cursor to the second line
        text.input(Some('o')).unwrap(); // o
        text.input(Some('n')).unwrap(); // on
        text.input(Some('e')).unwrap(); // one
        text.input(Some(' ')).unwrap(); // one 
        text.input(Some('t')).unwrap(); // one t (cursor now in second line)

        let lines: Vec<(isize, String)> = text.render_lines(
            |line_ctx| {
                Some((
                    line_ctx.active_line_offset,
                    line_ctx
                        .contents
                        .iter()
                        .map(|ctx| ctx.character.char)
                        .collect::<String>(),
                ))
            },
            LineRenderConfig::new(5).with_word_wrapping(false), // config
        );

        assert_eq!(lines.len(), 4);
        // Cursor is at position 5 (after "one t"), which is in line 1
        assert_eq!(lines[0], (-1, "one ".to_string())); // 1 line before cursor
        assert_eq!(lines[1], (0, "two ".to_string())); // cursor line - offset 0
        assert_eq!(lines[2], (1, "three ".to_string())); // 1 line after cursor
        assert_eq!(lines[3], (2, "four".to_string())); // 2 lines after cursor
    }

    #[test]
    fn test_render_lines_with_newlines() {
        let text = TypingSession::new("hello world\nthis is\na test").unwrap();

        let lines: Vec<String> = text.render_lines(
            |line_ctx| {
                Some(
                    line_ctx
                        .contents
                        .iter()
                        .map(|ctx| ctx.character.char)
                        .collect::<String>(),
                )
            },
            LineRenderConfig::new(20).with_newline_breaking(true), // config with newline breaking
        );

        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "hello world\n"); // newline is last character of line
        assert_eq!(lines[1], "this is\n"); // newline is last character of line  
        assert_eq!(lines[2], "a test"); // no trailing newline
    }

    #[test]
    fn test_render_lines_without_newline_breaking() {
        let text = TypingSession::new("hello world\nthis is").unwrap();

        let lines: Vec<String> = text.render_lines(
            |line_ctx| {
                Some(
                    line_ctx
                        .contents
                        .iter()
                        .map(|ctx| ctx.character.char)
                        .collect::<String>(),
                )
            },
            LineRenderConfig::new(20).with_newline_breaking(false), // config without newline breaking
        );

        // Should treat \n as regular character and not break
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0], "hello world\nthis is");
    }

    #[test]
    fn test_completion_percentage() {
        let mut text = TypingSession::new("hello").unwrap();

        // Initially 0% completed
        assert_eq!(text.completion_percentage(), 0.0);

        // Type first character - 20% completed
        text.input(Some('h')).unwrap();
        assert_eq!(text.completion_percentage(), 20.0);

        // Type second character - 40% completed
        text.input(Some('e')).unwrap();
        assert_eq!(text.completion_percentage(), 40.0);

        // Type remaining characters
        text.input(Some('l')).unwrap();
        text.input(Some('l')).unwrap();
        text.input(Some('o')).unwrap();

        // Should be 100% completed
        assert_eq!(text.completion_percentage(), 100.0);

        // Test with empty text (should return None, so we handle this case)
        if let Some(empty_text) = TypingSession::new("") {
            assert_eq!(empty_text.completion_percentage(), 0.0);
        }
    }

    #[test]
    fn test_words_typed_count() {
        let mut session = TypingSession::new("hello world test").unwrap();

        // Debug: print word boundaries and characters
        for i in 0..session.word_count() {
            if let Some(word) = session.get_word(i) {
                let chars: String = (word.start..word.end)
                    .map(|idx| session.get_character(idx).map(|c| c.char).unwrap_or('?'))
                    .collect();
                println!(
                    "Word {}: start={}, end={}, chars='{}'",
                    i, word.start, word.end, chars
                );
            }
        }

        // Print all characters with their positions
        for i in 0..session.text_len() {
            if let Some(ch) = session.get_character(i) {
                println!("Char {}: '{}'", i, ch.char);
            }
        }

        // Initially no words typed
        println!(
            "Initial: input_len={}, words_typed={}",
            session.input_len(),
            session.words_typed_count()
        );
        assert_eq!(session.words_typed_count(), 0);

        // Type "h" - still in first word
        session.input(Some('h')).unwrap();
        assert_eq!(session.words_typed_count(), 0);

        // Type "hell" - still in first word
        session.input(Some('e')).unwrap();
        session.input(Some('l')).unwrap();
        session.input(Some('l')).unwrap();
        assert_eq!(session.words_typed_count(), 0);

        // Type "hello" - completed first word but not past it
        session.input(Some('o')).unwrap();
        assert_eq!(session.words_typed_count(), 1);

        session.input(Some(' ')).unwrap();
        assert_eq!(session.words_typed_count(), 1);

        // Type "w" - starting second word
        session.input(Some('w')).unwrap();
        session.input(Some('o')).unwrap();
        assert_eq!(session.words_typed_count(), 1);

        // Type "world" - complete second word
        session.input(Some('r')).unwrap();
        session.input(Some('l')).unwrap();
        session.input(Some('d')).unwrap();
        assert_eq!(session.words_typed_count(), 2);

        // Type space after "world"
        session.input(Some(' ')).unwrap();
        assert_eq!(session.words_typed_count(), 2);

        // Type "t" - starting third word
        session.input(Some('t')).unwrap();
        assert_eq!(session.words_typed_count(), 2);

        // Complete "test"
        session.input(Some('e')).unwrap();
        session.input(Some('s')).unwrap();
        session.input(Some('t')).unwrap();
        assert_eq!(session.words_typed_count(), 3);

        // Test edge case: single word
        let mut single_word = TypingSession::new("hello").unwrap();
        assert_eq!(single_word.words_typed_count(), 0);

        // Type complete word
        for ch in "hello".chars() {
            single_word.input(Some(ch)).unwrap();
        }
        assert_eq!(single_word.words_typed_count(), 1);

        // Test edge case: text with leading/trailing spaces
        let mut spaced = TypingSession::new(" hello world ").unwrap();
        assert_eq!(spaced.words_typed_count(), 0);

        // Type the leading space
        spaced.input(Some(' ')).unwrap();
        assert_eq!(spaced.words_typed_count(), 0);

        // Type "hello"
        for ch in "hello".chars() {
            spaced.input(Some(ch)).unwrap();
        }
        assert_eq!(spaced.words_typed_count(), 1);

        // Type space after hello
        spaced.input(Some(' ')).unwrap();
        assert_eq!(spaced.words_typed_count(), 1);
    }
}
