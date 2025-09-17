use std::time::{Duration, Instant};

use ratatui::{
    style::{Color, Style},
    symbols::Marker,
    text::Span,
};
use serde::{Deserialize, Serialize};
use terminal_colorsaurus::QueryOptions;

const DEFAULT_SPINNER: [char; 8] = ['⣷', '⣯', '⣟', '⡿', '⢿', '⣻', '⣽', '⣾'];

/// General theme
#[derive(Debug, Deserialize, Serialize)]
pub struct Theme {
    pub spinner: Spinner,
    pub text: TextTheme,
    pub plot: PlotTheme,
    pub cursor: CursorTheme,
    pub term_fg: Color,
    pub term_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        let terminal_palette = terminal_colorsaurus::color_palette(QueryOptions::default()).ok();

        let (term_fg, term_bg) = if let Some(palette) = terminal_palette {
            let fg = palette.foreground.scale_to_8bit();
            let bg = palette.background.scale_to_8bit();
            (Color::Rgb(fg.0, fg.1, fg.2), Color::Rgb(bg.0, bg.1, bg.2))
        } else {
            (Color::White, Color::Black)
        };

        Self {
            spinner: Spinner::default(),
            text: TextTheme::default(),
            plot: PlotTheme::default(),
            cursor: CursorTheme::default(),
            term_fg,
            term_bg,
        }
    }
}

/// Spinner logic inspired from: https://crates.io/crates/throbber-widgets-tui
#[derive(Debug, Deserialize, Serialize)]
pub struct Spinner {
    pub color: Color,
    pub animation: Vec<char>,
    pub timing_millis: u64,
}

#[derive(Debug)]
pub struct SpinnerState {
    last_tick: Instant,
    index: u8,
    timing: Duration,
}

impl Default for Spinner {
    fn default() -> Self {
        Self {
            color: Color::Yellow,
            animation: Vec::from(DEFAULT_SPINNER),
            timing_millis: 250,
        }
    }
}

impl SpinnerState {
    pub fn tick(&mut self) {
        if self.last_tick.elapsed() > self.timing {
            self.index = self.index.checked_add(1).unwrap_or_default();
            self.last_tick = Instant::now();
        }
    }
}

impl Spinner {
    pub fn make_state(&self) -> SpinnerState {
        SpinnerState {
            last_tick: Instant::now(),
            index: 0,
            timing: Duration::from_millis(self.timing_millis),
        }
    }

    pub fn render(&self, state: &mut SpinnerState) -> Span<'_> {
        let len = self.animation.len() as u8;
        state.index %= len;
        let symbol = self.animation[state.index as usize];
        Span::styled(format!(" {symbol} "), Style::new().fg(self.color))
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct CursorTheme {
    pub color: Color,
    pub text: Color,
}

impl Default for CursorTheme {
    fn default() -> Self {
        Self {
            color: Color::White,
            text: Color::Black,
        }
    }
}

/// Text color theme
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TextTheme {
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub highlight: Color,
}

impl Default for TextTheme {
    fn default() -> Self {
        Self {
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
            highlight: Color::Blue,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum PlotSymbol {
    Dot,
    Block,
    HalfBlock,
    Braille,
    Bar,
}

impl PlotSymbol {
    /// Returns the marker that the symbol corresponds to.
    ///
    /// This doesn't use the `From` trait, as we can't make that a const fn
    pub const fn as_marker(self) -> Marker {
        match self {
            Self::Dot => Marker::Dot,
            Self::Bar => Marker::Bar,
            Self::Block => Marker::Block,
            Self::Braille => Marker::Braille,
            Self::HalfBlock => Marker::HalfBlock,
        }
    }
}

/// Plot color and symbol theme
#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PlotTheme {
    pub raw_wpm: Color,
    pub actual_wpm: Color,
    pub accuracy: Color,
    pub errors: Color,
    pub scatter_symbol: PlotSymbol,
    pub line_symbol: PlotSymbol,
}

impl Default for PlotTheme {
    fn default() -> Self {
        Self {
            raw_wpm: Color::Gray,
            actual_wpm: Color::Yellow,
            accuracy: Color::Gray,
            errors: Color::Red,
            scatter_symbol: PlotSymbol::Dot,
            line_symbol: PlotSymbol::HalfBlock,
        }
    }
}
