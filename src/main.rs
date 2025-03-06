use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{text::Text, DefaultTerminal, Frame, Terminal};
use smol_macros::main;

struct Library;

impl Library {
    pub async fn get_words(
        amount: usize,
        max_length: Option<usize>,
    ) -> Result<TypingSession, minreq::Error> {
        let max_length_param = if let Some(ml) = max_length {
            format!("?length={ml}")
        } else {
            String::new()
        };

        let words = minreq::get(format!(
            "https://random-word-api.herokuapp.com/word?number={amount}{max_length_param}"
        ))
        .send()?
        .json::<Vec<String>>()?
        .into_iter()
        .flat_map(|mut word| {
            word.push(' ');
            word.chars().collect::<Vec<_>>()
        })
        .collect();

        Ok(TypingSession {
            text: words,
            input: Vec::new(),
        })
    }
}

enum CharacterResult {
    Wrong(char), // TBD: Use character here to display multiple wrong characters after a word, like monkeytype does.
    Corrected,   // TBD: Support seeing if a character was typed wrong before, but is now corrected.
    Right,
}

struct TypingSession {
    text: Vec<char>,
    input: Vec<CharacterResult>,
}

impl TypingSession {
    fn handle_events(&mut self) -> std::io::Result<()> {
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => match key.code {
                // Add character
                KeyCode::Char(char) => {
                    let current = self.input.len();

                    if self.text[current] == char {
                        self.input.push(CharacterResult::Right);
                    } else {
                        self.input.push(CharacterResult::Wrong(char));
                    }
                }
                // Delete character
                KeyCode::Backspace => {
                    self.input.pop();
                }
                _ => {}
            },
            _ => {}
        }
        Ok(())
    }
}

main! {
    async fn main() {
        let mut terminal = ratatui::init();
        let result = run(&mut terminal);
        ratatui::restore();
        result.await.expect("crash")
    }
}

pub async fn run(terminal: &mut DefaultTerminal) -> std::io::Result<()> {
    let test = Library::get_words(3, None).await.expect("Error");

    Ok(())
}
