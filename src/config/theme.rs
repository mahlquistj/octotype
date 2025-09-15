use ratatui::{style::Color, symbols::Marker};
use serde::{Deserialize, Serialize};
use terminal_colorsaurus::QueryOptions;
use throbber_widgets_tui::Set;

/// General theme
#[derive(Debug, Deserialize, Serialize)]
pub struct Theme {
    pub spinner_color: Color,
    pub spinner_symbol: SpinnerSymbol,
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
            (Color::Rgb(255, 255, 255), Color::Rgb(0, 0, 0))
        };

        Self {
            spinner_color: Color::Yellow,
            spinner_symbol: SpinnerSymbol::BrailleSix,
            text: TextTheme::default(),
            plot: PlotTheme::default(),
            cursor: CursorTheme::default(),
            term_fg,
            term_bg,
        }
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

/// The different kinds of symbols available for the loading-screen spinner
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub enum SpinnerSymbol {
    Ascii,

    Arrow,
    DoubleArrow,

    BlackCircle,

    BoxDrawing,

    BrailleOne,
    BrailleDouble,
    BrailleSix,
    BrailleSixDouble,
    BrailleEight,
    BrailleEightDouble,

    Canadian,

    Clock,

    HorizontalBlock,

    OghamA,
    OghamB,
    OghamC,

    Paranthesis,

    QuadrantBlock,
    QuadrantBlockCrack,

    VerticalBlock,

    WhiteSquare,
    WhiteCircle,
}

impl SpinnerSymbol {
    /// Returns the set that the symbol corresponds to.
    ///
    /// This doesn't use the `From` trait, as we can't make that a const fn
    pub const fn as_set(&self) -> Set {
        use throbber_widgets_tui::*;
        match self {
            Self::Ascii => ASCII,
            Self::Arrow => ARROW,
            Self::DoubleArrow => DOUBLE_ARROW,
            Self::BlackCircle => BLACK_CIRCLE,
            Self::BoxDrawing => BOX_DRAWING,
            Self::BrailleOne => BRAILLE_ONE,
            Self::BrailleDouble => BRAILLE_DOUBLE,
            Self::BrailleSix => BRAILLE_SIX,
            Self::BrailleSixDouble => BRAILLE_SIX_DOUBLE,
            Self::BrailleEight => BRAILLE_EIGHT,
            Self::BrailleEightDouble => BRAILLE_EIGHT_DOUBLE,
            Self::Canadian => CANADIAN,
            Self::Clock => CLOCK,
            Self::HorizontalBlock => HORIZONTAL_BLOCK,
            Self::OghamA => OGHAM_A,
            Self::OghamB => OGHAM_B,
            Self::OghamC => OGHAM_C,
            Self::Paranthesis => PARENTHESIS,
            Self::QuadrantBlock => QUADRANT_BLOCK,
            Self::QuadrantBlockCrack => QUADRANT_BLOCK_CRACK,
            Self::VerticalBlock => VERTICAL_BLOCK,
            Self::WhiteSquare => WHITE_SQUARE,
            Self::WhiteCircle => WHITE_CIRCLE,
        }
    }
}
