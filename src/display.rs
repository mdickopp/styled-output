//! [`Display`] trait implementation for styled data.

use core::fmt::{self, Display, Formatter};

use crate::{RESET_STYLE, Style};

/// Displayable value with associated text style information.
///
/// The value must implement the [`Display`] trait. When `StyledDisplay` is formatted or converted
/// to a string, its value is wrapped in ANSI control sequences that cause it to be displayed in the
/// style represented by [`style`](Self::style) when it is written to a terminal that interprets
/// such sequences.
#[expect(clippy::exhaustive_structs)]
pub struct StyledDisplay<T>
where
    T: Display + ?Sized,
{
    /// The text style in which to display the value.
    pub style: Style,
    /// The value to display in the text style represented by [`style`](Self::style).
    pub value: T,
}

impl<T> Display for StyledDisplay<T>
where
    T: Display + ?Sized,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: Short-circuit if style is default (i.e., no styling).
        let mut buffer = Style::new_set_style_buffer();
        let set_style_str = self.style.set_style(&mut buffer);
        if set_style_str.is_empty() {
            Display::fmt(&self.value, f)
        } else {
            f.write_str(set_style_str)?;
            // TODO: Attempt to write `RESET_STYLE` if formatting fails.
            Display::fmt(&self.value, f)?;
            f.write_str(RESET_STYLE)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fmt::Write as _, io::Write as _};

    use crate::Color;

    use super::*;

    #[test]
    fn styled_default_style() {
        let styled = StyledDisplay {
            style: Style::default(),
            value: "foo",
        };
        let result = styled.to_string();
        assert_eq!(result, "foo");
    }

    #[test]
    fn styled_foreground_color_style() {
        let styled = StyledDisplay {
            style: Style {
                foreground_color: Color::Yellow,
                ..Default::default()
            },
            value: "foo",
        };
        let result = styled.to_string();
        assert_eq!(result, "\x1b[33mfoo\x1b[0m");
    }

    #[test]
    fn styled_write_to_string() {
        let styled = StyledDisplay {
            style: Style {
                foreground_color: Color::Yellow,
                ..Default::default()
            },
            value: "foo",
        };
        let mut result = String::new();
        write!(&mut result, ">{styled}<").expect("writing to String failed");
        assert_eq!(result, ">\x1b[33mfoo\x1b[0m<");
    }

    #[test]
    fn styled_write_to_vector() {
        let styled = StyledDisplay {
            style: Style {
                foreground_color: Color::Yellow,
                ..Default::default()
            },
            value: "foo",
        };
        let mut result = Vec::new();
        write!(&mut result, ">{styled}<").expect("writing to Vec failed");
        assert_eq!(result, b">\x1b[33mfoo\x1b[0m<");
    }

    #[test]
    fn styled_string_formatting() {
        let styled = StyledDisplay {
            style: Style {
                foreground_color: Color::Yellow,
                ..Default::default()
            },
            value: "foo",
        };
        let mut result = String::new();
        write!(&mut result, ">{styled:_>5}<").expect("writing to String failed");
        assert_eq!(result, ">\x1b[33m__foo\x1b[0m<");
    }

    #[test]
    fn styled_float_formatting() {
        let styled = StyledDisplay {
            style: Style {
                foreground_color: Color::Yellow,
                ..Default::default()
            },
            value: 17.5_f32,
        };
        let mut result = String::new();
        write!(&mut result, ">{styled:+.2}<").expect("writing to String failed");
        assert_eq!(result, ">\x1b[33m+17.50\x1b[0m<");
    }
}
