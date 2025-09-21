use crate::buffer::Buffer;
use crate::config::Configuration;
use crate::input_handler::InputHandler;
use crate::render::{LineContext, LineRenderConfig, RenderingContext, RenderingIterator};
use crate::statistics::{Statistics, TempStatistics};
use crate::statistics_tracker::StatisticsTracker;
use crate::{Character, CharacterResult, Word};
use web_time::Duration;

/// A typing session
#[derive(Debug, Clone)]
pub struct TypingSession {
    text_buffer: Buffer,
    input_handler: InputHandler,
    statistics: StatisticsTracker,
    config: Configuration,
}

impl TypingSession {
    pub fn new(string: &str) -> Option<Self> {
        let text_buffer = Buffer::new(string)?;

        Some(Self {
            text_buffer,
            input_handler: InputHandler::new(),
            statistics: StatisticsTracker::new(),
            config: Configuration::default(),
        })
    }

    /// Set configuration
    pub fn with_configuration(mut self, config: Configuration) -> Self {
        self.config = config;
        self
    }

    /// Get character at index
    pub fn get_character(&self, index: usize) -> Option<&Character> {
        self.text_buffer.get_character(index)
    }

    /// Get word containing index
    pub fn get_word_containing_index(&self, index: usize) -> Option<&Word> {
        self.text_buffer.get_word_containing(index)
    }

    /// Returns the amount of characters currently in the text.
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

    /// Returns the amount of characters currently in the input.
    pub fn input_len(&self) -> usize {
        self.input_handler.input_len()
    }

    /// Returns true if the text has been fully typed.
    pub fn is_fully_typed(&self) -> bool {
        self.input_handler
            .is_fully_typed(self.text_buffer.text_len())
    }

    /// Returns the completion percentage of the current [Buffer] as a f64
    pub fn completion_percentage(&self) -> f64 {
        let input_len = self.input_handler.input_len();
        let text_len = self.text_buffer.text_len();

        if text_len == 0 {
            return 0.0;
        }

        (input_len as f64 / text_len as f64) * 100.0
    }

    /// Returns the current time elapsed in seconds
    pub fn time_elapsed(&self) -> f64 {
        self.statistics
            .total_duration()
            .as_ref()
            .map(Duration::as_secs_f64)
            .unwrap_or(0.0)
    }

    /// Get the current statistics recorded for the text
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

                // If adding the next word would exceed the line length, break now
                if current_line_length + 1 + look_ahead_length > config.line_length {
                    lines.push((current_line_contexts, lines.len())); // Store line with its index
                    current_line_contexts = Vec::new();
                    current_line_length = 0;
                    continue; // Skip the space
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

    /// Type input into the text.
    ///
    /// If `Some(char)` is given, the char will be added to the input and returned with it's [CharacterResult].
    /// If `None` is given, the text will delete a character from the input and returned with
    /// `CharacterResult::Deleted`.
    ///
    /// Returns `None` if trying to delete and the input is empty or full (All text has
    /// been typed).
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

    /// Finalize the typing session and return the final Statistics
    /// This consumes the TypingSession and should only be called when typing is complete
    pub fn finalize(self) -> Result<Statistics, &'static str> {
        if !self.is_fully_typed() {
            return Err("Cannot finalize: typing session is not complete");
        }

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

        // Try to finalize
        let final_stats = text.finalize();
        assert!(final_stats.is_ok());

        let stats = final_stats.unwrap();
        // Verify the statistics contain expected data
        assert_eq!(stats.counters.adds, 2);
        assert_eq!(stats.counters.corrects, 2);
        assert_eq!(stats.counters.errors, 0);
    }

    #[test]
    fn test_finalization_before_completion() {
        let text = TypingSession::new("hello").unwrap();

        // Try to finalize without completing
        let result = text.finalize();
        assert!(result.is_err());
        if let Err(msg) = result {
            assert_eq!(msg, "Cannot finalize: typing session is not complete");
        }
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
        assert_eq!(lines[0], "hello");
        assert_eq!(lines[1], "world this");
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
        assert_eq!(lines[0], (0, "one".to_string())); // cursor line - offset 0
        assert_eq!(lines[1], (1, "two".to_string())); // 1 line after cursor
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
        assert_eq!(lines[0], (-1, "one".to_string())); // 1 line before cursor
        assert_eq!(lines[1], (0, "two".to_string())); // cursor line - offset 0
        assert_eq!(lines[2], (1, "three".to_string())); // 1 line after cursor
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
}
