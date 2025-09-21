mod config;
mod math;
mod statistics;
mod text;
mod text_buffer;
mod input_handler;
mod statistics_tracker;

pub use config::*;
pub use math::*;
pub use statistics::*;
pub use text::*;
pub use text_buffer::*;
pub use input_handler::*;
pub use statistics_tracker::*;

const AVERAGE_WORD_LENGTH: usize = 5;

// Shared types for readability
type Timestamp = f64;
type Minutes = f64;
type Float = f64;
