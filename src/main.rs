use crossterm::event::{self, Event};
use ratatui::{text::Text, Frame};
use smol_macros::main;

struct Library;

impl Library {
    pub async fn get_words(amount: usize, max_length: Option<usize>) -> Result<Vec<String>, minreq::Error> {
    
        let max_length_param = if let Some(ml) = max_length {
            format!("?length={ml}")
            } else {
String::new()
};
        minreq::get(format!("https://random-word-api.herokuapp.com/word?number={amount}{max_length_param}")).send()?.json() 
    }
}

struct TypingSession {
    text: Vec<char>,
    input: Vec<char>,
}

impl TypingSession {}

main! {
    async fn main() {
        let test = Library::get_words(3, None).await.expect("Error");

        println!("{test:?}");
    }
}
