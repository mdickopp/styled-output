//! Text style (color and attributes).

/// Text color.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Color {
    /// The color that the terminal displays by default.
    #[default]
    Default,
    /// Black.
    ///
    /// This color may be indistinguishable from [`DarkGray`](Self::DarkGray) in some terminal
    /// emulators.
    Black,
    /// Red.
    Red,
    /// Green.
    Green,
    /// Yellow.
    ///
    /// The actual color is implemented inconsistently in different terminal emulators, and may be a
    /// variant of brown, orange, yellow, olive, or greenish yellow. If it is important that the
    /// color is actually yellow, [`LightYellow`](Self::LightYellow) should be preferred.
    Yellow,
    /// Blue.
    Blue,
    /// Magenta.
    Magena,
    /// Cyan.
    Cyan,
    /// Gray.
    ///
    /// A light gray color, lighter than [`DarkGray`](Self::DarkGray).
    ///
    /// This color may be indistinguishable from [`White`](Self::White) in some terminal emulators.
    LightGray,
    /// Dark gray.
    ///
    /// A medium or dark gray color, darker than [`LightGray`](Self::LightGray).
    ///
    /// This color may be indistinguishable from [`Black`](Self::Black) in some terminal emulators.
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
    ///
    /// This color may be indistinguishable from [`LightGray`](Self::LightGray) in some terminal
    /// emulators.
    White,
}

impl Color {
    /// Returns the ANSI color code if the color is used for the foreground.
    #[inline]
    #[must_use]
    const fn foreground_code(self) -> &'static [u8] {
        match self {
            Self::Default => b"39",
            Self::Black => b"30",
            Self::Red => b"31",
            Self::Green => b"32",
            Self::Yellow => b"33",
            Self::Blue => b"34",
            Self::Magena => b"35",
            Self::Cyan => b"36",
            Self::LightGray => b"37",
            Self::DarkGray => b"90",
            Self::LightRed => b"91",
            Self::LightGreen => b"92",
            Self::LightYellow => b"93",
            Self::LightBlue => b"94",
            Self::LightMagenta => b"95",
            Self::LightCyan => b"96",
            Self::White => b"97",
        }
    }

    /// Returns the ANSI color code if the color is used for the background.
    #[inline]
    #[must_use]
    const fn background_code(self) -> &'static [u8] {
        match self {
            Self::Default => b"49",
            Self::Black => b"40",
            Self::Red => b"41",
            Self::Green => b"42",
            Self::Yellow => b"43",
            Self::Blue => b"44",
            Self::Magena => b"45",
            Self::Cyan => b"46",
            Self::LightGray => b"47",
            Self::DarkGray => b"100",
            Self::LightRed => b"101",
            Self::LightGreen => b"102",
            Self::LightYellow => b"103",
            Self::LightBlue => b"104",
            Self::LightMagenta => b"105",
            Self::LightCyan => b"106",
            Self::White => b"107",
        }
    }
}

/// Text color and attributes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
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
    pub blinking: bool,
}

impl Style {
    // FIXME: should be pub(crate)
    /// Writes the ANSI control sequence that sets this style to the specified buffer and returns a
    /// subslice containing the control sequence.
    ///
    /// # Panics
    ///
    /// Panics if `buffer` is too small. It should have a size of at least 15 bytes.
    pub fn set_style(self, buffer: &mut [u8]) -> &[u8] {
        // Stores the Control Sequence Introducer (CSI) in the buffer if it is empty, otherwise
        // appends a semicolon to the buffer. Updates the number of bytes stored in the buffer.
        #[inline]
        fn append_prefix(buffer: &mut [u8], n: &mut usize) {
            if *n == 0 {
                append_slice(buffer, n, b"\x1b[");
            } else {
                append_byte(buffer, n, b';');
            }
        }

        // Appends a byte to the buffer and updates the number of bytes stored in the buffer.
        #[inline]
        fn append_byte(buffer: &mut [u8], n: &mut usize, byte: u8) {
            buffer[*n] = byte;
            *n += 1;
        }

        // Appends a slice to the buffer and updates the number of bytes stored in the buffer.
        #[inline]
        fn append_slice(buffer: &mut [u8], n: &mut usize, slice: &[u8]) {
            let len = slice.len();
            buffer[*n..*n + len].copy_from_slice(slice);
            *n += len;
        }

        assert!(buffer.len() >= 15, "buffer too small");

        // Number of bytes stored in the buffer.
        let mut n = 0;

        if self.foreground_color != Color::Default {
            append_prefix(buffer, &mut n);
            append_slice(buffer, &mut n, self.foreground_color.foreground_code());
        }

        if self.background_color != Color::Default {
            append_prefix(buffer, &mut n);
            append_slice(buffer, &mut n, self.background_color.background_code());
        }

        if self.bold {
            append_prefix(buffer, &mut n);
            append_byte(buffer, &mut n, b'1');
        }

        if self.underlined {
            append_prefix(buffer, &mut n);
            append_byte(buffer, &mut n, b'4');
        }

        if self.blinking {
            append_prefix(buffer, &mut n);
            append_byte(buffer, &mut n, b'5');
        }

        if n != 0 {
            append_byte(buffer, &mut n, b'm');
        }

        &buffer[..n]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_style_default() {
        let style = Style::default();
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert!(buffer.is_empty());
    }

    #[test]
    fn set_style_foreground_color() {
        let mut style = Style::default();
        style.foreground_color = Color::Yellow;
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert_eq!(b"\x1b[33m", buffer);
    }

    #[test]
    fn set_style_background_color() {
        let mut style = Style::default();
        style.background_color = Color::LightMagenta;
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert_eq!(b"\x1b[105m", buffer);
    }

    #[test]
    fn set_style_foreground_and_background_color() {
        let mut style = Style::default();
        style.foreground_color = Color::White;
        style.background_color = Color::Blue;
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert_eq!(b"\x1b[97;44m", buffer);
    }

    #[test]
    fn set_style_bold() {
        let mut style = Style::default();
        style.bold = true;
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert_eq!(b"\x1b[1m", buffer);
    }

    #[test]
    fn set_style_underlined() {
        let mut style = Style::default();
        style.underlined = true;
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert_eq!(b"\x1b[4m", buffer);
    }

    #[test]
    fn set_style_blinking() {
        let mut style = Style::default();
        style.blinking = true;
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert_eq!(b"\x1b[5m", buffer);
    }

    #[test]
    fn set_style_all() {
        let style = Style {
            foreground_color: Color::Cyan,
            background_color: Color::DarkGray,
            bold: true,
            underlined: true,
            blinking: true,
        };
        let mut buffer = [0_u8; 15];
        let buffer = style.set_style(&mut buffer);
        assert_eq!(b"\x1b[36;100;1;4;5m", buffer);
    }

    #[test]
    #[should_panic(expected = "buffer too small")]
    fn set_style_buffer_too_small() {
        let style = Style {
            foreground_color: Color::Cyan,
            background_color: Color::DarkGray,
            bold: true,
            underlined: true,
            blinking: true,
        };
        let mut buffer = [0_u8; 14];
        style.set_style(&mut buffer);
    }
}
