use serde::Deserialize;

use super::SourceError;

/// Wrapper for parsing quotes from quotes-api
#[derive(Deserialize)]
struct QuoteWrapper {
    quote: Quote,
}

/// A quote object from quotes-api
#[derive(Deserialize)]
#[serde(rename = "quote")]
struct Quote {
    // id: String,
    content: String,
    // author: String,
    // slug: String,
    // length: usize,
    // tags: Vec<String>,
}

pub fn get_words() -> Result<Vec<String>, SourceError> {
    let words = minreq::get("https://api.quotable.kurokeita.dev/api/quotes/random")
        .send()?
        .json::<QuoteWrapper>()?
        .quote
        .content
        .split_ascii_whitespace()
        .map(str::to_string)
        .collect::<Vec<String>>();

    Ok(words)
}
