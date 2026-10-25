use codemap::SpanLoc;
use std::fmt::Debug;

/// A trait to allow replacing logging mechanisms
pub trait Logger: Debug {
    /// Logs message from a [`@debug`](https://sass-lang.com/documentation/at-rules/debug/)
    /// statement
    fn debug(&self, location: SpanLoc, message: &str);

    /// Logs message from a [`@warn`](https://sass-lang.com/documentation/at-rules/warn/)
    /// statement
    fn warn(&self, location: SpanLoc, message: &str);
}

/// Logs events to standard error, through [`eprintln!`]
#[derive(Debug)]
pub struct StdLogger;

impl Logger for StdLogger {
    #[inline]
    fn debug(&self, location: SpanLoc, message: &str) {
        eprintln!(
            "{}:{} DEBUG: {}",
            location.file.name(),
            location.begin.line + 1,
            message
        );
    }

    #[inline]
    fn warn(&self, location: SpanLoc, message: &str) {
        eprintln!(
            "Warning: {}\n    ./{}:{}:{}",
            message,
            location.file.name(),
            location.begin.line + 1,
            location.begin.column + 1
        );
    }
}

/// Discards all logs
#[derive(Debug)]
pub struct NullLogger;

impl Logger for NullLogger {
    #[inline]
    fn debug(&self, _location: SpanLoc, _message: &str) {}

    #[inline]
    fn warn(&self, _location: SpanLoc, _message: &str) {}
}
