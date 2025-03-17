use ratatui::{style::Color, symbols::Marker};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TextTheme {
    pub color: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Default for TextTheme {
    fn default() -> Self {
        Self {
            color: Color::White,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Symbol {
    #[serde(alias = "dot")]
    Dot,
    #[serde(alias = "block")]
    Block,
    #[serde(alias = "halfblock")]
    HalfBlock,
    #[serde(alias = "braille")]
    Braille,
    #[serde(alias = "bar")]
    Bar,
}

impl From<Symbol> for Marker {
    fn from(symbol: Symbol) -> Marker {
        match symbol {
            Symbol::Dot => Self::Dot,
            Symbol::Bar => Self::Bar,
            Symbol::Block => Self::Block,
            Symbol::Braille => Self::Braille,
            Symbol::HalfBlock => Self::HalfBlock,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PlotTheme {
    pub raw_wpm: Color,
    pub actual_wpm: Color,
    pub accurracy: Color,
    pub errors: Color,
    pub scatter_symbol: Symbol,
    pub line_symbol: Symbol,
}

impl Default for PlotTheme {
    fn default() -> Self {
        Self {
            raw_wpm: Color::Gray,
            actual_wpm: Color::Yellow,
            accurracy: Color::Gray,
            errors: Color::Red,
            scatter_symbol: Symbol::Dot,
            line_symbol: Symbol::HalfBlock,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Theme {
    pub spinner: Color,
    pub text: TextTheme,
    pub plot: PlotTheme,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            spinner: Color::Yellow,
            text: TextTheme::default(),
            plot: PlotTheme::default(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub theme: Theme,
}
