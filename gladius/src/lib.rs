mod config;
mod math;
mod statistics;
mod text;

pub use config::*;
pub use math::*;
pub use statistics::*;
pub use text::*;

const AVERAGE_WORD_LENGTH: usize = 5;

// Shared types for readability
type Timestamp = f64;
type Minutes = f64;
type Float = f64;
