use std::cmp::Ordering;

#[derive(Default)]
pub enum CharacterState {
    // Pre delete or add
    #[default]
    None,

    // Post deleted
    WasWrong,
    WasCorrected,

    // Post added
    Wrong,
    Corrected,
    Correct,
}

pub struct Word<'a> {
    pub chars: &'a [char], // Borrow this instead
    pub is_misspelled: bool,
}

pub struct Character {
    pub char: char,
    pub state: CharacterState,
}

pub struct CharacterContext<'a> {
    pub word: Word<'a>,
    pub character: &'a Character,
}

pub trait Renderer<Output> {
    fn render_char(character: &Character) -> Output;
}

pub struct Text {
    characters: Vec<Character>,
    words: Vec<(usize, usize)>,
}

impl Text {
    pub fn render<Output, R: Renderer<Output>>(&self) -> Vec<Output> {
        self.characters
            .iter()
            .map(R::render_char)
            .collect::<Vec<Output>>()
    }
}
