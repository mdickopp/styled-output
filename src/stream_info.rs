//! Information about standard output and standard error.

use std::{
    env,
    sync::atomic::{AtomicI8, AtomicI32, AtomicU8, Ordering},
};

use terminal_size::Width;

/// Raw line width value indicating that the raw line width has not yet been determined.
const RAW_LINE_WIDTH_UNKNOWN: i32 = -2;

/// Raw line width value indicating that a stream does not refer to a terminal, or the terminal
/// width cannot be determined.
const RAW_LINE_WIDTH_NONE: i32 = -1;

/// Default line width.
///
/// This value is returned by [`StreamInfo::line_width`] if the stream does not refer to a terminal,
/// or the terminal width cannot be determined.
pub const DEFAULT_LINE_WIDTH: u16 = 80;

/// Information about standard output.
///
/// Use this [`StreamInfo`] instance to query information about the standard output stream or set
/// its color mode.
pub static STDOUT_INFO: StreamInfo<private::TerminalSizeStdout> =
    StreamInfo::new(private::TerminalSizeStdout);

/// Information about standard error.
///
/// Use this [`StreamInfo`] instance to query information about the standard error stream or set its
/// color mode.
pub static STDERR_INFO: StreamInfo<private::TerminalSizeStderr> =
    StreamInfo::new(private::TerminalSizeStderr);

/// Whether to use colors and other styling.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorMode {
    /// Determine automatically whether to use colors and other styling.
    ///
    /// Colors and styling are used if the stream refers to a terminal, unless the environment
    /// variable `NO_COLOR` is set to a non-empty value.
    ///
    /// See [`StreamInfo::use_color`] for the exact rules that determine color usage.
    #[default]
    Auto,
    /// Do not use colors and other styling, irrespective of whether the stream refers to a terminal
    /// or whether the environment variable `NO_COLOR` is set.
    Never,
    /// Use colors and other styling, irrespective of whether the stream refers to a terminal or
    /// whether the environment variable `NO_COLOR` is set.
    Always,
}

/// Information about a stream.
///
/// Two static instances of this structure are available, [`STDOUT_INFO`] and [`STDERR_INFO`],
/// providing information about the standard output and standard error streams, respectively.
pub struct StreamInfo<T: private::TerminalSize> {
    /// Returns the terminal size of the stream.
    terminal_size: T,
    /// Whether to use colors and other styling.
    ///
    /// The value corresponds to the discriminant value of [`ColorMode`] cast to `u8`.
    raw_color_mode: AtomicU8,
    /// Raw line width.
    ///
    /// The value is either the line width (which has type `u16`) cast to `i32`,
    /// [`RAW_LINE_WIDTH_UNKNOWN`], or [`RAW_LINE_WIDTH_NONE`].
    raw_line_width: AtomicI32,
}

impl<T: private::TerminalSize> StreamInfo<T> {
    /// Returns an object that provides information about a stream.
    const fn new(terminal_size: T) -> Self {
        Self {
            terminal_size,
            raw_color_mode: AtomicU8::new(ColorMode::Auto as isize as u8),
            raw_line_width: AtomicI32::new(RAW_LINE_WIDTH_UNKNOWN),
        }
    }

    /// Returns whether colors and other styling should be used when writing to the stream.
    ///
    /// The result is determined according to these rules:
    /// - If the color mode has been set to [`ColorMode::Never`] with [`set_color_mode`], `false` is
    ///   returned.
    /// - Otherwise, if the color mode has been set to [`ColorMode::Always`] with
    ///   [`set_color_mode`], `true` is returned.
    /// - Otherwise, if the environment variable `NO_COLOR` is set to a non-empty value, `false` is
    ///   returned.
    /// - Otherwise, if the stream refers to a terminal, `true` is returned.
    /// - Otherwise, `false` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// use styled_output::stream_info::STDOUT_INFO;
    ///
    /// if STDOUT_INFO.use_color() {
    ///     println!("Should use color on standard output.");
    /// } else {
    ///     println!("Should not use color on standard output.");
    /// }
    /// ```
    ///
    /// [`set_color_mode`]: Self::set_color_mode
    // TODO: Example that actually generates color output.
    #[must_use]
    pub fn use_color(&self) -> bool {
        let mut color_mode = self.raw_color_mode.load(Ordering::Acquire);
        if color_mode == ColorMode::Auto as isize as u8 {
            color_mode = if !env_no_color() && self.get_raw_line_width() != RAW_LINE_WIDTH_NONE {
                ColorMode::Always as isize as u8
            } else {
                ColorMode::Never as isize as u8
            };
            self.raw_color_mode.store(color_mode, Ordering::Relaxed);
        }
        color_mode == ColorMode::Always as isize as u8
    }

    /// Sets whether colors and other styling should be used when writing to the stream.
    ///
    /// If the color mode is set to [`ColorMode::Auto`] (which is the default if it is not set
    /// explicitly with this method.), the usage of colors depends on whether the stream refers to a
    /// terminal and whether the environment variable `NO_COLOR` is set. Otherwise,
    /// [`ColorMode::Never`] disables color usage, and [`ColorMode::Always`] enables it.
    ///
    /// See [`use_color`] for the exact rules that determine color usage.
    ///
    /// # Example
    ///
    /// ```
    /// use styled_output::stream_info::{ColorMode, STDOUT_INFO};
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

    /// Returns the line width that should be used for word wrapping when writing to the stream.
    ///
    /// Returns the terminal width if the stream refers to a terminal, or a default line width
    /// ([`DEFAULT_LINE_WIDTH`]) otherwise.
    ///
    /// # Example
    ///
    /// ```
    /// use styled_output::stream_info::STDOUT_INFO;
    ///
    /// println!("The line width for standard output is {}.", STDOUT_INFO.line_width());
    /// ```
    #[must_use]
    pub fn line_width(&self) -> u16 {
        let raw_line_width = self.get_raw_line_width();
        if raw_line_width >= 0 {
            raw_line_width as u16
        } else {
            DEFAULT_LINE_WIDTH
        }
    }

    /// Returns the raw line width of the stream.
    ///
    /// The raw line width is either the line width (which has type `u16`) cast to `i32`,
    /// [`RAW_LINE_WIDTH_UNKNOWN`], or [`RAW_LINE_WIDTH_NONE`].
    #[must_use]
    fn get_raw_line_width(&self) -> i32 {
        let mut raw_line_width = self.raw_line_width.load(Ordering::Relaxed);
        if raw_line_width == RAW_LINE_WIDTH_UNKNOWN {
            raw_line_width = if let Some((Width(width), _)) = self.terminal_size.terminal_size() {
                width as i32
            } else {
                RAW_LINE_WIDTH_NONE
            };
            self.raw_line_width.store(raw_line_width, Ordering::Relaxed);
        }
        raw_line_width
    }
}

/// Value indicating that the value of [`ENV_NO_COLOR`] has not yet been determined.
const ENV_NO_COLOR_UNKNOWN: i8 = -1;

/// Flag whether the `NO_COLOR` environment variable is set to a non-empty value.
///
/// The value is either a `bool` cast to `i8`, or [`ENV_NO_COLOR_UNKNOWN`].
static ENV_NO_COLOR: AtomicI8 = AtomicI8::new(ENV_NO_COLOR_UNKNOWN);

/// Returns whether the `NO_COLOR` environment variable is set to a non-empty value.
fn env_no_color() -> bool {
    let mut env_no_color = ENV_NO_COLOR.load(Ordering::Relaxed);
    if env_no_color == ENV_NO_COLOR_UNKNOWN {
        env_no_color = !env::var_os("NO_COLOR").is_none_or(|value| value.is_empty()) as i8;
        ENV_NO_COLOR.store(env_no_color, Ordering::Relaxed);
    }
    env_no_color != false as i8
}

/// Private module containing implementation details.
mod private {
    #[cfg(any(unix, windows))]
    use std::io;

    #[cfg(any(unix, windows))]
    use terminal_size;
    use terminal_size::{Height, Width};

    /// Returns the terminal size of a stream.
    pub trait TerminalSize {
        /// Returns the terminal size of the stream.
        #[must_use]
        fn terminal_size(&self) -> Option<(Width, Height)>;
    }

    /// Returns the terminal size of the standard output stream.
    pub struct TerminalSizeStdout;

    #[cfg(any(unix, windows))]
    impl TerminalSize for TerminalSizeStdout {
        #[inline]
        fn terminal_size(&self) -> Option<(Width, Height)> {
            terminal_size::terminal_size_of(io::stdout())
        }
    }

    #[cfg(not(any(unix, windows)))]
    impl TerminalSize for TerminalSizeStdout {
        #[inline]
        fn terminal_size(&self) -> Option<(Width, Height)> {
            None
        }
    }

    /// Returns the terminal size of the standard error stream.
    pub struct TerminalSizeStderr;

    #[cfg(any(unix, windows))]
    impl TerminalSize for TerminalSizeStderr {
        #[inline]
        fn terminal_size(&self) -> Option<(Width, Height)> {
            terminal_size::terminal_size_of(io::stderr())
        }
    }

    #[cfg(not(any(unix, windows)))]
    impl TerminalSize for TerminalSizeStderr {
        #[inline]
        fn terminal_size(&self) -> Option<(Width, Height)> {
            None
        }
    }
}

#[cfg(all(any(test, doc), target_os = "linux"))]
mod tests {
    use std::{
        env,
        fs::OpenOptions,
        io::{Error, Result},
        os::fd::{AsFd, BorrowedFd, FromRawFd, OwnedFd, RawFd},
        ptr::{null, null_mut},
        sync::{Mutex, MutexGuard},
    };

    use terminal_size::terminal_size_of;

    use super::*;

    impl<'fd> private::TerminalSize for BorrowedFd<'fd> {
        #[inline]
        fn terminal_size(&self) -> Option<(Width, terminal_size::Height)> {
            terminal_size_of(self)
        }
    }

    /// Sets or removes the environment variable `NO_COLOR`.
    ///
    /// Also resets the cached flag whether the environment variable is set to a non-empty value
    /// ([`ENV_NO_COLOR`]).
    ///
    /// # Safety
    ///
    /// Callers must retain the returned [`MutexGuard`] object as long as environment variables may
    /// be accessed (read or modified).
    unsafe fn set_env_no_color(value: Option<&'static str>) -> MutexGuard<'static, ()> {
        static ENV_MUTEX: Mutex<()> = Mutex::new(());
        let env_guard = ENV_MUTEX.lock().unwrap_or_else(|e| {
            ENV_MUTEX.clear_poison();
            e.into_inner()
        });
        ENV_NO_COLOR.store(ENV_NO_COLOR_UNKNOWN, Ordering::SeqCst);
        match value {
            // SAFETY: Access to environment variables is protected by `env_guard`.
            Some(value) => unsafe {
                env::set_var("NO_COLOR", value);
            },
            // SAFETY: Access to environment variables is protected by `env_guard`.
            None => unsafe {
                env::remove_var("NO_COLOR");
            },
        };
        env_guard
    }

    /// Opens a terminal and sets its width to the specified value.
    ///
    /// Returns a tuple containing the master and slave file descriptors, respectively, or an error.
    fn open_term(width: u16) -> Result<(OwnedFd, OwnedFd)> {
        let mut master_raw_fd: RawFd = -1;
        let mut slave_raw_fd: RawFd = -1;
        let winsize = libc::winsize {
            ws_row: 25,
            ws_col: width,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        // SAFETY: If the function returns successfully, the resources it creates, `master_raw_fd`
        // and `slave_raw_fd`, are converted to `OwnedFd`. If it returns an error, the error is
        // returned to the caller.
        if unsafe {
            libc::openpty(
                &mut master_raw_fd,
                &mut slave_raw_fd,
                null_mut(),
                null(),
                &winsize,
            )
        } == -1
        {
            return Err(Error::last_os_error());
        }
        // SAFETY: Both `master_raw_fd` and `slave_raw_fd` are open file descriptors. No other
        // reference to these file descriptors exists.
        Ok(unsafe {
            (
                OwnedFd::from_raw_fd(master_raw_fd),
                OwnedFd::from_raw_fd(slave_raw_fd),
            )
        })
    }

    #[test]
    fn test_use_color_no_terminal() {
        for env_no_color in [None, Some(""), Some("0"), Some("1")] {
            for color_mode in [
                None,
                Some(ColorMode::Auto),
                Some(ColorMode::Never),
                Some(ColorMode::Always),
            ] {
                for multiple_calls in [false, true] {
                    // SAFETY: `_env_guard` is retained as long as environment variables may be
                    // accessed.
                    let _env_guard = unsafe { set_env_no_color(env_no_color) };
                    let file = OpenOptions::new()
                        .write(true)
                        .open("/dev/null")
                        .expect("cannot open /dev/null for writing");
                    let stream_info = StreamInfo::new(file.as_fd());
                    if let Some(color_mode) = color_mode {
                        stream_info.set_color_mode(color_mode);
                    }

                    let expected_use_color = match color_mode {
                        Some(ColorMode::Always) => true,
                        _ => false,
                    };
                    if multiple_calls {
                        let _ = stream_info.use_color();
                    }
                    assert_eq!(
                        stream_info.use_color(),
                        expected_use_color,
                        "env_no_color = {env_no_color:?}, color_mode = {color_mode:?}, multiple_calls = {multiple_calls:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn test_use_color_terminal() {
        for env_no_color in [None, Some(""), Some("0"), Some("1")] {
            for color_mode in [
                None,
                Some(ColorMode::Auto),
                Some(ColorMode::Never),
                Some(ColorMode::Always),
            ] {
                for multiple_calls in [false, true] {
                    // SAFETY: `_env_guard` is retained as long as environment variables may be
                    // accessed.
                    let _env_guard = unsafe { set_env_no_color(env_no_color) };
                    let term = open_term(80).expect("cannot open pseudoterminal");
                    let stream_info = StreamInfo::new(term.1.as_fd());
                    if let Some(color_mode) = color_mode {
                        stream_info.set_color_mode(color_mode);
                    }

                    let expected_use_color = match (env_no_color, color_mode) {
                        (_, Some(ColorMode::Never)) => false,
                        (_, Some(ColorMode::Always)) => true,
                        (None | Some(""), _) => true,
                        _ => false,
                    };
                    if multiple_calls {
                        let _ = stream_info.use_color();
                    }
                    assert_eq!(
                        stream_info.use_color(),
                        expected_use_color,
                        "env_no_color = {env_no_color:?}, color_mode = {color_mode:?}, multiple_calls = {multiple_calls:?}",
                    );
                }
            }
        }
    }

    #[test]
    fn test_line_width_no_terminal() {
        for multiple_calls in [false, true] {
            let file = OpenOptions::new()
                .write(true)
                .open("/dev/null")
                .expect("cannot open /dev/null for writing");
            let stream_info = StreamInfo::new(file.as_fd());

            if multiple_calls {
                let _ = stream_info.line_width();
            }
            assert_eq!(
                stream_info.line_width(),
                DEFAULT_LINE_WIDTH,
                "multiple_calls = {multiple_calls:?}",
            );
        }
    }

    #[test]
    fn test_line_width_terminal() {
        for line_width in [1, 20, DEFAULT_LINE_WIDTH, 112] {
            for multiple_calls in [false, true] {
                let term = open_term(line_width).expect("cannot open pseudoterminal");
                let stream_info = StreamInfo::new(term.1.as_fd());

                if multiple_calls {
                    let _ = stream_info.line_width();
                }
                assert_eq!(
                    stream_info.line_width(),
                    line_width,
                    "line_width = {line_width}, multiple_calls = {multiple_calls:?}",
                );
            }
        }
    }
}
