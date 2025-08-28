use serde::Deserialize;

/// Wrapper for parsing quotes from quotes-api
#[derive(Deserialize)]
pub struct QuoteWrapper {
    pub quote: Quote,
}

/// A quote object from quotes-api
#[derive(Deserialize)]
#[serde(rename = "quote")]
pub struct Quote {
    // id: String,
    pub content: String,
    // author: String,
    // slug: String,
    // length: usize,
    // tags: Vec<String>,
}
