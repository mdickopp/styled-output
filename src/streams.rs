//! Information about output standard streams (standard output and standard error).

use std::{
    env,
    io::{Stderr, Stdout},
    marker::PhantomData,
    sync::atomic::{AtomicI32, AtomicU8, Ordering},
};

use terminal_size::Width;

/// Raw line width value that is used to indicate that the raw line width has not yet been
/// determined.
const RAW_LINE_WIDTH_UNKNOWN: i32 = -2;

/// Raw line width value that is used to indicate that the output stream does not refer to a
/// terminal, or the terminal width cannot be determined.
const RAW_LINE_WIDTH_NONE: i32 = -1;

/// Default line width.
///
/// This value is returned by [`OutputStreamInfo::line_width`] if the stream does not refer to a
/// terminal, or the terminal width cannot be determined.
pub const DEFAULT_LINE_WIDTH: u16 = 80;

/// Information about standard output.
///
/// Use this static [`OutputStreamInfo`] instance to query information about the standard output
/// stream or set its color mode.
pub static STDOUT_INFO: OutputStreamInfo<Stdout> = OutputStreamInfo::new();

/// Information about standard error.
///
/// Use this static [`OutputStreamInfo`] instance to query information about the standard error
/// stream or set its color mode.
pub static STDERR_INFO: OutputStreamInfo<Stderr> = OutputStreamInfo::new();

/// Whether to use colors and other styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorMode {
    /// Determine automatically whether to use colors and other styling.
    ///
    /// Colors and styling are used if the output stream refers to a terminal, unless the
    /// environment variable `NO_COLOR` is set to a non-empty value.
    Auto,
    /// Do not use colors and other styling, irrespective of whether the output stream refers to a
    /// terminal or whether the environment variable `NO_COLOR` is set.
    Never,
    /// Use colors and other styling, irrespective of whether the output stream refers to a terminal
    /// or whether the environment variable `NO_COLOR` is set.
    Always,
}

/// Information about an output standard stream (standard output or standard error).
///
/// Two static instances of this structure are available, [`STDOUT_INFO`] and [`STDERR_INFO`],
/// containing information about the standard output and standard error streams, respectively.
pub struct OutputStreamInfo<S: private::StdStream> {
    /// Whether to use colors and other styling.
    ///
    /// The value corresponds to the discriminant value of [`ColorMode`] cast to `u8`.
    raw_color_mode: AtomicU8,
    /// Raw line width.
    ///
    /// The value corresponds to either the line width (which has type `u16`) cast to `i32`,
    /// [`RAW_LINE_WIDTH_UNKNOWN`], or [`RAW_LINE_WIDTH_NONE`].
    raw_line_width: AtomicI32,
    /// Phantom data to mark `S` as used.
    phantom: PhantomData<S>,
}

impl<S: private::StdStream> OutputStreamInfo<S> {
    /// Returns information about an output standard stream for which no information has been
    /// determined yet.
    const fn new() -> Self {
        Self {
            raw_color_mode: AtomicU8::new(ColorMode::Auto as isize as u8),
            raw_line_width: AtomicI32::new(RAW_LINE_WIDTH_UNKNOWN),
            phantom: PhantomData,
        }
    }

    /// Returns whether colors and other styling should be used on the output stream.
    ///
    /// The result is determined according to these rules:
    /// - If the color mode has been set to [`ColorMode::Never`] with [`set_color_mode`], `false` is
    ///   returned.
    /// - Otherwise, if the color mode has been set to [`ColorMode::Always`] with
    ///   [`set_color_mode`], `true` is returned.
    /// - Otherwise, if the environment variable `NO_COLOR` is set to a non-empty value, `false` is
    ///   returned.
    /// - Otherwise, if the output stream refers to a terminal, `true` is returned.
    /// - Otherwise, `false` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use styled_output::streams::STDOUT_INFO;
    ///
    /// if STDOUT_INFO.use_color() {
    ///     println!("Should use color on standard output.");
    /// } else {
    ///     println!("Should not use color on standard output.");
    /// }
    /// ```
    ///
    /// [`set_color_mode`]: Self::set_color_mode
    // TODO: Exmamle that actually generates color output.
    pub fn use_color(&self) -> bool {
        let mut color_mode = self.raw_color_mode.load(Ordering::Acquire);
        if color_mode == ColorMode::Auto as isize as u8 {
            // TODO: Cache value of NO_COLOR?
            color_mode = if env::var_os("NO_COLOR").is_none_or(|value| value.is_empty())
                && self.get_raw_line_width() != RAW_LINE_WIDTH_NONE
            {
                ColorMode::Always as isize as u8
            } else {
                ColorMode::Never as isize as u8
            };
            self.raw_color_mode.store(color_mode, Ordering::Relaxed);
        }
        color_mode == ColorMode::Always as isize as u8
    }

    /// Sets whether colors and other styling should be used on the output stream.
    ///
    /// If the color mode is set to [`ColorMode::Auto`] (which is the default if it is not set
    /// explicitly with this method.), the usage of colors depends on whether the output stream
    /// refers to a terminal and whether the environment variable `NO_COLOR` is set. Otherwise,
    /// [`ColorMode::Never`] disables color usage, and [`ColorMode::Always`] enables it.
    ///
    /// See [`use_color`] for the exact rules that determine color usage.
    ///
    /// # Example
    ///
    /// ```
    /// use styled_output::streams::{ColorMode, STDOUT_INFO};
    ///
    /// STDOUT_INFO.set_color_mode(ColorMode::Never);
    /// assert_eq!(STDOUT_INFO.use_color(), false);
    ///
    /// STDOUT_INFO.set_color_mode(ColorMode::Always);
    /// assert_eq!(STDOUT_INFO.use_color(), true);
    /// ```
    ///
    /// [`use_color`]: Self::use_color
    pub fn set_color_mode(&self, color_mode: ColorMode) {
        self.raw_color_mode
            .store(color_mode as isize as u8, Ordering::Release);
    }

    /// Returns the line width that should be used for word wrapping when writing to the output
    /// stream.
    ///
    /// Returns the terminal width if the output stream refers to a terminal, or a default line
    /// width ([`DEFAULT_LINE_WIDTH`]) otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use styled_output::streams::STDOUT_INFO;
    ///
    /// println!("The line width for standard output is {}.", STDOUT_INFO.line_width());
    /// ```
    pub fn line_width(&self) -> u16 {
        let raw_line_width = self.get_raw_line_width();
        if raw_line_width >= 0 {
            raw_line_width as u16
        } else {
            DEFAULT_LINE_WIDTH
        }
    }

    /// Returns the raw line width of the output stream.
    ///
    /// The raw line width corresponds to either the line width (which has type `u16`) cast to
    /// `i32`, [`RAW_LINE_WIDTH_UNKNOWN`], or [`RAW_LINE_WIDTH_NONE`].
    fn get_raw_line_width(&self) -> i32 {
        let mut line_width = self.raw_line_width.load(Ordering::Relaxed);
        if line_width == RAW_LINE_WIDTH_UNKNOWN {
            line_width =
                if let Some((Width(width), _)) = terminal_size::terminal_size_of(S::get_stream()) {
                    width as i32
                } else {
                    RAW_LINE_WIDTH_NONE
                };
            self.raw_line_width.store(line_width, Ordering::Relaxed);
        }
        line_width
    }
}

/// Private module containing implementation details.
mod private {
    use std::{
        io::{self, Stderr, Stdout},
        os::fd::AsFd,
    };

    /// Provides access to a standard stream instance.
    pub trait StdStream {
        /// The standard stream type.
        type Stream: AsFd;

        /// Returns the standard stream instance.
        #[must_use]
        fn get_stream() -> Self::Stream;
    }

    impl StdStream for Stdout {
        type Stream = Stdout;

        #[inline]
        #[must_use]
        fn get_stream() -> Stdout {
            io::stdout()
        }
    }

    impl StdStream for Stderr {
        type Stream = Stderr;

        #[inline]
        #[must_use]
        fn get_stream() -> Stderr {
            io::stderr()
        }
    }
}
