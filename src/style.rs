//! Text color and style.

/// Text color.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// The color that the terminal displays by default.
    #[default]
    Default,
    /// Black.
    Black,
    /// Red.
    Red,
    /// Green.
    Green,
    /// Yellow.
    Yellow,
    /// Blue.
    Blue,
    /// Magenta.
    Magena,
    /// Cyan.
    Cyan,
    /// Gray.
    Gray,
    /// Dark gray.
    DarkGray,
    /// Light red.
    LightRed,
    /// Light green.
    LightGreen,
    /// Light yellow.
    LightYellow,
    /// Light blue.
    LightBlue,
    /// Light magenta.
    LightMagenta,
    /// Light cyan.
    LightCyan,
    /// White.
    White,
}

/// Text color and style.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style {
    /// Foreground color.
    pub foreground_color: Color,
    /// Background color.
    pub background_color: Color,
    /// Bold text.
    pub bold: bool,
    /// Underlined text.
    pub underlined: bool,
    /// Blinking text.
    pub blink: bool,
}
