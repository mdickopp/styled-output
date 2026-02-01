//! Text style (color and attributes).

use core::{mem::MaybeUninit, slice};

/// ANSI control sequence that resets all styling.
pub(crate) const RESET_STYLE: &str = "\x1b[0m";

/// Text color.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
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
    const fn foreground_code(self) -> &'static str {
        match self {
            Self::Default => "39",
            Self::Black => "30",
            Self::Red => "31",
            Self::Green => "32",
            Self::Yellow => "33",
            Self::Blue => "34",
            Self::Magena => "35",
            Self::Cyan => "36",
            Self::LightGray => "37",
            Self::DarkGray => "90",
            Self::LightRed => "91",
            Self::LightGreen => "92",
            Self::LightYellow => "93",
            Self::LightBlue => "94",
            Self::LightMagenta => "95",
            Self::LightCyan => "96",
            Self::White => "97",
        }
    }

    /// Returns the ANSI color code if the color is used for the background.
    #[inline]
    #[must_use]
    const fn background_code(self) -> &'static str {
        match self {
            Self::Default => "49",
            Self::Black => "40",
            Self::Red => "41",
            Self::Green => "42",
            Self::Yellow => "43",
            Self::Blue => "44",
            Self::Magena => "45",
            Self::Cyan => "46",
            Self::LightGray => "47",
            Self::DarkGray => "100",
            Self::LightRed => "101",
            Self::LightGreen => "102",
            Self::LightYellow => "103",
            Self::LightBlue => "104",
            Self::LightMagenta => "105",
            Self::LightCyan => "106",
            Self::White => "107",
        }
    }
}

/// Text color and attributes.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
#[expect(clippy::exhaustive_structs)]
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
    /// Creates a buffer to be passed to the [`set_style`](Self::set_style) function.
    #[inline]
    #[must_use]
    pub(crate) fn new_set_style_buffer() -> [MaybeUninit<u8>; 15] {
        [const { MaybeUninit::uninit() }; 15]
    }

    /// Writes the ANSI control sequence that sets this style to the specified buffer and returns a
    /// string containing the control sequence.
    pub(crate) fn set_style(self, buffer: &mut [MaybeUninit<u8>; 15]) -> &str {
        // Stores the Control Sequence Introducer (CSI) in the buffer if it is empty, otherwise
        // appends a semicolon to the buffer. Updates the number of bytes stored in the buffer.
        #[inline]
        fn push_prefix(buffer: &mut [MaybeUninit<u8>; 15], len: &mut usize) {
            if *len == 0 {
                push_str(buffer, len, "\x1b[");
            } else {
                push_ascii(buffer, len, b';');
            }
        }

        // Appends an ASCII character to the buffer and updates the number of bytes stored in the
        // buffer.
        #[inline]
        fn push_ascii(buffer: &mut [MaybeUninit<u8>; 15], len: &mut usize, ch: u8) {
            assert!(ch.is_ascii());
            buffer[*len].write(ch);
            *len += 1;
        }

        // Appends a string slice to the buffer and updates the number of bytes stored in the
        // buffer.
        #[inline]
        fn push_str(buffer: &mut [MaybeUninit<u8>; 15], len: &mut usize, string: &str) {
            let string_ptr = string.as_bytes().as_ptr();
            let string_len = string.len();
            // SAFETY: `string` is reconstructed from its original raw pointer and length, so merely
            // its type is changed. Furthermore, `MaybeUninit<u8>` is guaranteed to have the same
            // size, alignment, and ABI as `u8`.
            let src =
                unsafe { slice::from_raw_parts(string_ptr as *const MaybeUninit<u8>, string_len) };
            buffer[*len..*len + string_len].copy_from_slice(src);
            *len += string_len;
        }

        // Number of bytes stored in the buffer.
        let mut len = 0;

        if self.foreground_color != Color::Default {
            push_prefix(buffer, &mut len);
            push_str(buffer, &mut len, self.foreground_color.foreground_code());
        }

        if self.background_color != Color::Default {
            push_prefix(buffer, &mut len);
            push_str(buffer, &mut len, self.background_color.background_code());
        }

        if self.bold {
            push_prefix(buffer, &mut len);
            push_ascii(buffer, &mut len, b'1');
        }

        if self.underlined {
            push_prefix(buffer, &mut len);
            push_ascii(buffer, &mut len, b'4');
        }

        if self.blinking {
            push_prefix(buffer, &mut len);
            push_ascii(buffer, &mut len, b'5');
        }

        if len != 0 {
            push_ascii(buffer, &mut len, b'm');
        }

        // SAFETY: `len` tracks the number of of bytes stored in the buffer, so all elements in the
        // resulting slice are initialized.
        let b = unsafe { slice::from_raw_parts(buffer.as_ptr() as *const u8, len) };
        // SAFETY: Only ASCII characters and `str` slices have been appended to the buffer.
        // Therefore, the buffer is guaranteed to contain valid UTF-8.
        unsafe { str::from_utf8_unchecked(b) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_style_default() {
        let style = Style::default();
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert!(result.is_empty());
    }

    #[test]
    fn set_style_foreground_color() {
        let style = Style {
            foreground_color: Color::Yellow,
            ..Default::default()
        };
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert_eq!(result, "\x1b[33m");
    }

    #[test]
    fn set_style_background_color() {
        let style = Style {
            background_color: Color::LightMagenta,
            ..Default::default()
        };
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert_eq!(result, "\x1b[105m");
    }

    #[test]
    fn set_style_foreground_and_background_color() {
        let style = Style {
            foreground_color: Color::White,
            background_color: Color::Blue,
            ..Default::default()
        };
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert_eq!(result, "\x1b[97;44m");
    }

    #[test]
    fn set_style_bold() {
        let style = Style {
            bold: true,
            ..Default::default()
        };
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert_eq!(result, "\x1b[1m");
    }

    #[test]
    fn set_style_underlined() {
        let style = Style {
            underlined: true,
            ..Default::default()
        };
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert_eq!(result, "\x1b[4m");
    }

    #[test]
    fn set_style_blinking() {
        let style = Style {
            blinking: true,
            ..Default::default()
        };
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert_eq!(result, "\x1b[5m");
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
        let mut buffer = Style::new_set_style_buffer();
        let result = style.set_style(&mut buffer);
        assert_eq!(result, "\x1b[36;100;1;4;5m");
    }
}
