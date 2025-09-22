//! # Render Module - Text Display and Line Management
//!
//! This module provides data structures and utilities for rendering typing trainer text
//! to user interfaces. It handles text line breaking, cursor positioning, and provides
//! contextual information needed for visual display and styling.
//!
//! ## Key Features
//!
//! - **Flexible Line Breaking**: Word-aware and character-based line wrapping
//! - **Cursor Tracking**: Maintains cursor position across line boundaries
//! - **Context-Rich Rendering**: Provides character, word, and cursor information
//! - **Configurable Display**: Customizable line length and breaking behavior
//!
//! ## Rendering Pipeline
//!
#![doc = simple_mermaid::mermaid!("../diagrams/rendering_pipeline.mmd")]
//!
//! ## Usage Example
//!
//! ```rust
//! use gladius::render::{LineRenderConfig, LineContext};
//! use gladius::session::TypingSession;
//!
//! let session = TypingSession::new("hello world this is a test").unwrap();
//! let config = LineRenderConfig::new(10).with_word_wrapping(false);
//!
//! let lines = session.render_lines(|line_context: LineContext| {
//!     // Process each line and return your line representation
//!     Some(format!("Line: {} chars", line_context.contents.len()))
//! }, config);
//! ```

use crate::{Character, TypingSession, Word};

/// Context information for rendering a single character
///
/// Provides all the information needed to render one character, including its
/// typing state, containing word context, cursor position, and text index.
/// Used by UI frameworks to determine styling, highlighting, and visual effects.
///
/// # Fields
///
/// - `character`: The character data including its char value and typing state
/// - `word`: The word containing this character (None for whitespace)
/// - `has_cursor`: Whether the typing cursor is currently at this position
/// - `index`: Zero-based index of this character in the full text
#[derive(Debug, Clone)]
pub struct RenderingContext<'a> {
    /// The character being rendered with its current typing state
    pub character: &'a Character,
    /// The word containing this character (None for whitespace between words)
    pub word: Option<&'a Word>,
    /// Whether the typing cursor is positioned at this character
    pub has_cursor: bool,
    /// Position of this character in the full text (zero-based)
    pub index: usize,
}

/// Context information for rendering a complete line of text
///
/// Groups multiple characters into a line with metadata about the line's
/// relationship to the cursor position. Used by line-based rendering systems
/// to display text with proper line breaks and cursor tracking.
///
/// # Fields
///
/// - `active_line_offset`: Distance from the line containing the cursor
/// - `contents`: All characters in this line with their rendering contexts
///
/// # Line Offset Examples
///
/// ```text
/// Line -1: "hello world"     (offset: -1, above cursor line)
/// Line  0: "this |is text"   (offset:  0, contains cursor)
/// Line +1: "more text"       (offset: +1, below cursor line)
/// ```
#[derive(Debug, Clone)]
pub struct LineContext<'a> {
    /// Offset from the line containing the cursor (0 = cursor line, -1 = above, +1 = below)
    pub active_line_offset: isize,
    /// All characters in this line with their complete rendering contexts
    pub contents: Vec<RenderingContext<'a>>,
}

/// Configuration for line rendering behavior
///
/// Controls how text is broken into lines for display. Provides options for
/// line length limits, word wrapping behavior, and newline handling to support
/// different UI layouts and display requirements.
///
/// # Breaking Behavior
///
/// - **Word Wrapping**: When disabled, tries to break at word boundaries
/// - **Character Wrapping**: When word wrapping enabled, breaks anywhere
/// - **Newline Breaking**: When enabled, forces line breaks at `\n` characters
///
/// # Usage Examples
///
/// ```rust
/// use gladius::render::LineRenderConfig;
///
/// // Basic configuration: 80 characters, break at words
/// let config = LineRenderConfig::new(80);
///
/// // Allow breaking words mid-word for narrow displays
/// let narrow_config = LineRenderConfig::new(20)
///     .with_word_wrapping(true);
///
/// // Ignore newlines for continuous text flow
/// let flow_config = LineRenderConfig::new(50)
///     .with_newline_breaking(false);
/// ```
#[derive(Debug, Clone)]
pub struct LineRenderConfig {
    /// Maximum number of characters per line before wrapping
    pub line_length: usize,
    /// Whether to allow breaking words in the middle (vs. only at word boundaries)
    pub wrap_words: bool,
    /// Whether to force line breaks at newline characters (\n)
    pub break_at_newlines: bool,
}

impl LineRenderConfig {
    /// Create a new line rendering configuration with default settings
    ///
    /// Sets up line breaking with the specified character limit and sensible defaults:
    /// - Word wrapping disabled (prefers breaking at word boundaries)
    /// - Newline breaking enabled (respects `\n` characters)
    ///
    /// # Parameters
    ///
    /// * `line_length` - Maximum characters per line before wrapping
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::render::LineRenderConfig;
    ///
    /// let config = LineRenderConfig::new(80); // 80-character lines
    /// assert_eq!(config.line_length, 80);
    /// assert_eq!(config.wrap_words, false);
    /// assert_eq!(config.break_at_newlines, true);
    /// ```
    pub fn new(line_length: usize) -> Self {
        Self {
            line_length,
            wrap_words: false,
            break_at_newlines: true,
        }
    }

    /// Configure word wrapping behavior (builder pattern)
    ///
    /// Controls whether lines can break in the middle of words or only at word boundaries.
    ///
    /// # Parameters
    ///
    /// * `wrap_words` - If true, allows breaking words; if false, breaks only at spaces
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::render::LineRenderConfig;
    ///
    /// // Break anywhere for narrow displays
    /// let config = LineRenderConfig::new(20).with_word_wrapping(true);
    ///
    /// // Preserve word boundaries for readability
    /// let config = LineRenderConfig::new(80).with_word_wrapping(false);
    /// ```
    pub fn with_word_wrapping(mut self, wrap_words: bool) -> Self {
        self.wrap_words = wrap_words;
        self
    }

    /// Configure newline character handling (builder pattern)
    ///
    /// Controls whether newline characters (`\n`) force line breaks or are treated as
    /// regular whitespace for continuous text flow.
    ///
    /// # Parameters
    ///
    /// * `break_at_newlines` - If true, `\n` forces line breaks; if false, ignores them
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gladius::render::LineRenderConfig;
    ///
    /// // Respect paragraph breaks in source text
    /// let config = LineRenderConfig::new(80).with_newline_breaking(true);
    ///
    /// // Continuous flow ignoring paragraph breaks
    /// let config = LineRenderConfig::new(80).with_newline_breaking(false);
    /// ```
    pub fn with_newline_breaking(mut self, break_at_newlines: bool) -> Self {
        self.break_at_newlines = break_at_newlines;
        self
    }
}

/// Iterator that produces rendering contexts for each character in a typing session
///
/// Provides a convenient way to iterate through all characters in the text with
/// their complete rendering context including typing state, word association,
/// and cursor position. Used as the foundation for all rendering operations.
///
/// # Performance
///
/// - Iteration: O(1) per character
/// - Memory: O(1) iterator state (does not copy text data)
/// - Length: O(1) via ExactSizeIterator
///
/// # Example
///
/// ```rust
/// use gladius::session::TypingSession;
///
/// let session = TypingSession::new("hello world").unwrap();
/// let mut contexts: Vec<_> = session.render_iter().collect();
///
/// assert_eq!(contexts.len(), 11); // "hello world" = 11 chars
/// assert_eq!(contexts[0].character.char, 'h');
/// assert_eq!(contexts[0].index, 0);
/// assert!(contexts[0].has_cursor); // Cursor starts at position 0
/// ```
#[derive(Debug)]
pub struct RenderingIterator<'a> {
    /// Reference to the typing session being rendered
    typing_session: &'a TypingSession,
    /// Current character index in the iteration
    index: usize,
    /// Position of the typing cursor in the text
    cursor_position: usize,
}

impl<'a> From<&'a TypingSession> for RenderingIterator<'a> {
    /// Create a rendering iterator from a typing session
    ///
    /// Initializes the iterator at the beginning of the text with the cursor
    /// position set to the current input length of the session.
    fn from(value: &'a TypingSession) -> Self {
        Self {
            cursor_position: value.input_len(),
            index: 0,
            typing_session: value,
        }
    }
}

impl<'a> ExactSizeIterator for RenderingIterator<'a> {}

impl<'a> std::iter::FusedIterator for RenderingIterator<'a> {}

impl<'a> Iterator for RenderingIterator<'a> {
    type Item = RenderingContext<'a>;

    /// Get the next character's rendering context
    ///
    /// Returns a complete RenderingContext for the next character in the text,
    /// including its typing state, containing word, cursor position, and index.
    /// Returns None when all characters have been processed.
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.typing_session.text_len() {
            return None;
        }

        let character = self.typing_session.get_character(self.index)?;
        let word = self.typing_session.get_word_containing_index(self.index);
        let has_cursor = self.index == self.cursor_position;

        let context = RenderingContext {
            character,
            word,
            has_cursor,
            index: self.index,
        };

        self.index += 1;
        Some(context)
    }

    /// Get the exact number of remaining characters
    ///
    /// Returns precise bounds for the number of characters remaining in the iteration.
    /// Both lower and upper bounds are the same since text length is known.
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.typing_session.text_len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}
