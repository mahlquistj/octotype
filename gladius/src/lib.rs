mod statistics;
mod text;

pub use statistics::*;
pub use text::*;

const AVERAGE_WORD_LENGTH: usize = 5;

// Types for more general type-safety
type Timestamp = f64;
type Minutes = f64;

#[cfg(feature = "f64")]
type Float = f64;

#[cfg(not(feature = "f64"))]
type Float = f32;

// Get the minutes elapsed from a timestamp
pub(crate) fn minutes(timestamp: Timestamp) -> Minutes {
    timestamp / 60.0
}
