use crate::{Character, TypingSession, Word};

pub struct RenderingContext<'a> {
    pub character: &'a Character,
    pub word: Option<&'a Word>,
    pub has_cursor: bool,
    pub index: usize,
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
