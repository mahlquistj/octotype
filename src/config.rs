use ratatui::style::Color;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct TextColors {
    pub color: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}

impl Default for TextColors {
    fn default() -> Self {
        Self {
            color: Color::White,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::Red,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct PlotColors {
    pub raw_wpm: Color,
    pub actual_wpm: Color,
    pub accurracy: Color,
    pub errors: Color,
}

impl Default for PlotColors {
    fn default() -> Self {
        Self {
            raw_wpm: Color::Gray,
            actual_wpm: Color::Yellow,
            accurracy: Color::Gray,
            errors: Color::Red,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Theme {
    pub spinner: Color,
    pub text: TextColors,
    pub plot: PlotColors,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            spinner: Color::Yellow,
            text: TextColors::default(),
            plot: PlotColors::default(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Config {
    pub theme: Theme,
}
