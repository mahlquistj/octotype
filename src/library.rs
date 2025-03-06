use crate::TypingSession;

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

        let mut words: Vec<char> = minreq::get(format!(
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

        words.pop();

        Ok(TypingSession::new(words))
    }
}
