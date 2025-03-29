//! Text color and style.

use std::io::{self, Write};

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
    pub blinking: bool,
}

impl Color {
    /// Returns the ANSI color code if the color is used for the background.
    fn foreground_code(&self) -> &'static [u8] {
        match self {
            Color::Default => "39".as_bytes(),
            Color::Black => "30".as_bytes(),
            Color::Red => "31".as_bytes(),
            Color::Green => "32".as_bytes(),
            Color::Yellow => "33".as_bytes(),
            Color::Blue => "34".as_bytes(),
            Color::Magena => "35".as_bytes(),
            Color::Cyan => "36".as_bytes(),
            Color::Gray => "37".as_bytes(),
            Color::DarkGray => "90".as_bytes(),
            Color::LightRed => "91".as_bytes(),
            Color::LightGreen => "92".as_bytes(),
            Color::LightYellow => "93".as_bytes(),
            Color::LightBlue => "94".as_bytes(),
            Color::LightMagenta => "95".as_bytes(),
            Color::LightCyan => "96".as_bytes(),
            Color::White => "97".as_bytes(),
        }
    }

    /// Returns the ANSI color code if the color is used for the background.
    fn background_code(&self) -> &'static [u8] {
        match self {
            Color::Default => "49".as_bytes(),
            Color::Black => "40".as_bytes(),
            Color::Red => "41".as_bytes(),
            Color::Green => "42".as_bytes(),
            Color::Yellow => "43".as_bytes(),
            Color::Blue => "44".as_bytes(),
            Color::Magena => "45".as_bytes(),
            Color::Cyan => "46".as_bytes(),
            Color::Gray => "47".as_bytes(),
            Color::DarkGray => "100".as_bytes(),
            Color::LightRed => "101".as_bytes(),
            Color::LightGreen => "102".as_bytes(),
            Color::LightYellow => "103".as_bytes(),
            Color::LightBlue => "104".as_bytes(),
            Color::LightMagenta => "105".as_bytes(),
            Color::LightCyan => "106".as_bytes(),
            Color::White => "107".as_bytes(),
        }
    }
}

impl Style {
    /// Writes the ANSI control sequence that sets the current style.
    pub(crate) fn write_set_style<W>(&self, writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        let mut have_written = false;

        if self.foreground_color != Color::Default {
            Self::write_ansi_code(
                writer,
                self.foreground_color.foreground_code(),
                &mut have_written,
            )?;
        }

        if self.background_color != Color::Default {
            Self::write_ansi_code(
                writer,
                self.background_color.background_code(),
                &mut have_written,
            )?;
        }

        if self.bold {
            Self::write_ansi_code(writer, "1".as_bytes(), &mut have_written)?;
        }

        if self.underlined {
            Self::write_ansi_code(writer, "4".as_bytes(), &mut have_written)?;
        }

        if self.blinking {
            Self::write_ansi_code(writer, "5".as_bytes(), &mut have_written)?;
        }

        if have_written {
            writer.write_all("m".as_bytes())?;
        }

        Ok(())
    }

    /// Writes the ANSI control sequence that resets styling.
    pub(crate) fn write_reset_style<W>(writer: &mut W) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all("\x1b[0m".as_bytes())
    }

    /// Writes an ANSI code preceded by the Control Sequence Introducer (CSI) or a semicolon,
    /// depending on whether a previous part of the ANSI control sequence has been written.
    #[inline]
    fn write_ansi_code<W>(
        writer: &mut W,
        code: &'static [u8],
        have_written: &mut bool,
    ) -> io::Result<()>
    where
        W: ?Sized + Write,
    {
        writer.write_all(if *have_written {
            ";".as_bytes()
        } else {
            "\x1b[".as_bytes()
        })?;
        *have_written = true;
        writer.write_all(code)
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use super::*;

    #[test]
    fn test_write_set_style_default() {
        let style = Style::default();
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        assert!(buffer.is_empty());
    }

    #[test]
    fn test_write_set_style_fg_color() {
        let mut style = Style::default();
        style.foreground_color = Color::Yellow;
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[33m", written);
    }

    #[test]
    fn test_write_set_style_bg_color() {
        let mut style = Style::default();
        style.background_color = Color::LightMagenta;
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[105m", written);
    }

    #[test]
    fn test_write_set_style_fg_and_bg_color() {
        let mut style = Style::default();
        style.foreground_color = Color::White;
        style.background_color = Color::Blue;
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[97;44m", written);
    }

    #[test]
    fn test_write_set_style_bold() {
        let mut style = Style::default();
        style.bold = true;
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[1m", written);
    }

    #[test]
    fn test_write_set_style_underlined() {
        let mut style = Style::default();
        style.underlined = true;
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[4m", written);
    }

    #[test]
    fn test_write_set_style_blinking() {
        let mut style = Style::default();
        style.blinking = true;
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[5m", written);
    }

    #[test]
    fn test_write_set_style_all() {
        let style = Style {
            foreground_color: Color::Cyan,
            background_color: Color::DarkGray,
            bold: true,
            underlined: true,
            blinking: true,
        };
        let mut buffer = Vec::new();
        style
            .write_set_style(&mut buffer)
            .expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[36;100;1;4;5m", written);
    }

    #[test]
    fn test_write_reset_style() {
        let mut buffer = Vec::new();
        Style::write_reset_style(&mut buffer).expect("write to memory failed");
        let written = str::from_utf8(&buffer).expect("not valid UTF-8");
        assert_eq!("\x1b[0m", written);
    }
}
