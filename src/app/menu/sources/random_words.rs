use super::SourceError;

pub fn get_words(
    lang: Option<&String>,
    amount: u32,
    max_length: Option<u32>,
) -> Result<Vec<String>, SourceError> {
    let mut req = minreq::get("https://random-word-api.herokuapp.com/word")
        .with_param("number", amount.to_string());

    if let Some(language) = lang {
        req = req.with_param("lang", language);
    }

    if let Some(ml) = max_length {
        req = req.with_param("length", ml.to_string());
    }

    let words = req.send()?.json::<Vec<String>>()?;

    Ok(words)
}
