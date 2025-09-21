mod config;
mod input_handler;
mod math;
mod statistics;
mod statistics_tracker;
mod text;
mod text_buffer;

pub use config::*;
pub use input_handler::*;
pub use math::*;
pub use statistics::*;
pub use statistics_tracker::*;
pub use text::*;
pub use text_buffer::*;

const AVERAGE_WORD_LENGTH: usize = 5;

// Shared types for readability
type Timestamp = f64;
type Minutes = f64;
type Float = f64;
