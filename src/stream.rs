//! Styled output stream and writer.

use std::{
    io::{self, Stderr, Stdout, Write},
    sync::{Mutex, MutexGuard},
};

/// Output stream for styled output.
pub struct StyledStream<L: private::LockableStream> {
    inner: L,
}

impl<L: private::LockableStream> StyledStream<L> {
    /// Write a string to the stream.
    pub fn write(&self, s: &str) -> io::Result<()> {
        self.inner.lock().write_all(s.as_bytes())
    }
}

impl StyledStream<Stdout> {
    /// Returns a styled output stream for standard output.
    pub fn stdout() -> Self {
        Self {
            inner: io::stdout(),
        }
    }
}

impl StyledStream<Stderr> {
    /// Returns a styled output stream for standard error.
    pub fn stderr() -> Self {
        Self {
            inner: io::stderr(),
        }
    }
}

impl<W: Write> StyledStream<LockableWriter<W>> {
    /// Returns a styled output stream for a writer.
    pub fn from_writer(w: W) -> Self {
        Self {
            inner: LockableWriter {
                mutex: Mutex::new(w),
            },
        }
    }
}

struct LockableWriter<W: ?Sized + Write> {
    mutex: Mutex<W>,
}

struct StreamLock<'w, W: 'w + ?Sized + Write> {
    inner: MutexGuard<'w, W>,
}

impl<W: ?Sized + Write> Write for StreamLock<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        self.inner.write_all(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

mod private {
    use std::io::{Stderr, StderrLock, Stdout, StdoutLock, Write};

    use super::{LockableWriter, StreamLock};

    /// A lockable output stream handle.
    ///
    /// Locking the stream handle returns a guard that can be used to write to the stream, while also
    /// protecting the stream from concurrent writing.
    pub trait LockableStream {
        type Writer<'a>: Write
        where
            Self: 'a;

        fn lock(&self) -> Self::Writer<'_>;
    }

    impl LockableStream for Stdout {
        type Writer<'a> = StdoutLock<'static>;

        fn lock(&self) -> Self::Writer<'_> {
            self.lock()
        }
    }

    impl LockableStream for Stderr {
        type Writer<'a> = StderrLock<'static>;

        fn lock(&self) -> Self::Writer<'_> {
            self.lock()
        }
    }

    impl<W: ?Sized + Write> LockableStream for LockableWriter<W> {
        type Writer<'a>
            = StreamLock<'a, W>
        where
            Self: 'a;

        fn lock(&self) -> Self::Writer<'_> {
            StreamLock {
                inner: self.mutex.lock().unwrap_or_else(|e| {
                    self.mutex.clear_poison();
                    e.into_inner()
                }),
            }
        }
    }
}
