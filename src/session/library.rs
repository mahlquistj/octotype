use crate::session::TypingSession;
use super::Segment;

/// Utility struct for fetching words
pub struct Library;

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
        .json::<Vec<String>>()?;

        let last_segment = words.len() / 4;

        let words = words
            .chunks(4)
            .enumerate()
            .map(|(idx, words)| {
                let mut string = words
                    .iter()
                    .cloned()
                    .map(|mut word| {
                        word.push(' ');
                        word
                    })
                    .collect::<String>();

                if idx == last_segment {
                    string.pop();
                }

                Segment::from_iter(string.chars())
            })
            .collect();

        Ok(TypingSession::new(words))
    }
}
