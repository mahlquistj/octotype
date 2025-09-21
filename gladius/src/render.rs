use crate::{Character, TypingSession, Word};

pub struct RenderingContext<'a> {
    pub character: &'a Character,
    pub word: Option<&'a Word>,
    pub has_cursor: bool,
    pub index: usize,
}

pub struct LineContext<'a> {
    pub active_line_offset: isize,
    pub contents: Vec<RenderingContext<'a>>,
}

/// Configuration for line rendering behavior
pub struct LineRenderConfig {
    /// Maximum number of characters per line
    pub line_length: usize,
    /// Whether to allow breaking words in the middle
    pub wrap_words: bool,
    /// Whether to break at newline characters (\n)
    pub break_at_newlines: bool,
}

impl LineRenderConfig {
    pub fn new(line_length: usize) -> Self {
        Self {
            line_length,
            wrap_words: false,
            break_at_newlines: true,
        }
    }

    pub fn with_word_wrapping(mut self, wrap_words: bool) -> Self {
        self.wrap_words = wrap_words;
        self
    }

    pub fn with_newline_breaking(mut self, break_at_newlines: bool) -> Self {
        self.break_at_newlines = break_at_newlines;
        self
    }
}

/// Iterator for rendering contexts
pub struct RenderingIterator<'a> {
    typing_session: &'a TypingSession,
    index: usize,
    cursor_position: usize,
}

impl<'a> From<&'a TypingSession> for RenderingIterator<'a> {
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.typing_session.text_len().saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}
