//! Styled text.

use std::io::{self, Write};

use crate::Style;

/// Text that may have associated styling information.
pub trait StyledText<W: ?Sized + Write> {
    /// Writes the text without it styling information.
    fn write_unstyled(&self, writer: &mut W) -> io::Result<()>;

    /// Writes the styled text.
    fn write_styled(&self, writer: &mut W) -> io::Result<()>;
}

impl<W: ?Sized + Write> StyledText<W> for str {
    #[inline]
    fn write_unstyled(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.as_bytes())
    }

    #[inline]
    fn write_styled(&self, writer: &mut W) -> io::Result<()> {
        self.write_unstyled(writer)
    }
}

/// Owned text and styling information.
#[derive(Debug)]
pub struct StyledString {
    /// The text style.
    pub style: Style,
    /// The text.
    pub text: String,
}

impl<W: ?Sized + Write> StyledText<W> for StyledString {
    #[inline]
    fn write_unstyled(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.text.as_bytes())
    }

    #[inline]
    fn write_styled(&self, writer: &mut W) -> io::Result<()> {
        self.style.write_set_style(writer)?;
        self.write_unstyled(writer)?;
        Style::write_reset_style(writer)
    }
}

/// Styled text that references a string slice.
#[derive(Debug)]
pub struct StyledStr {
    /// The text style.
    pub style: Style,
    /// The text.
    pub text: str,
}

impl<W: ?Sized + Write> StyledText<W> for StyledStr {
    #[inline]
    fn write_unstyled(&self, writer: &mut W) -> io::Result<()> {
        writer.write_all(self.text.as_bytes())
    }

    #[inline]
    fn write_styled(&self, writer: &mut W) -> io::Result<()> {
        self.style.write_set_style(writer)?;
        self.write_unstyled(writer)?;
        Style::write_reset_style(writer)
    }
}
