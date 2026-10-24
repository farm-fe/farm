use std::{
    error::Error,
    fmt::{self, Display},
    io,
    string::FromUtf8Error,
    sync::Arc,
};

use codemap::{Span, SpanLoc};

pub type SassResult<T> = Result<T, Box<SassError>>;

/// `SassError`s can be either a structured error specific to `grass` or an
/// `io::Error`.
///
/// In the former case, the best way to interact with the error is to simply print
/// it to the user. The `Display` implementation of this kind of error mirrors
/// that of the errors `dart-sass` emits, e.g.
///```scss
/// Error: $number: foo is not a number.
///     |
/// 308 |     color: unit(foo);
///     |                 ^^^
///     |
/// ./input.scss:308:17
///```
///
#[derive(Debug, Clone)]
pub struct SassError {
    kind: SassErrorKind,
}

impl SassError {
    #[must_use]
    pub fn kind(self) -> PublicSassErrorKind {
        match self.kind {
            SassErrorKind::ParseError {
                message,
                loc,
                unicode,
            } => PublicSassErrorKind::ParseError {
                message,
                loc,
                unicode,
            },
            SassErrorKind::FromUtf8Error(s) => PublicSassErrorKind::FromUtf8Error(s),
            SassErrorKind::IoError(io) => PublicSassErrorKind::IoError(io),
            SassErrorKind::Raw(..) => unreachable!("raw errors should not be accessible by users"),
        }
    }

    pub(crate) fn raw(self) -> (String, Span) {
        match self.kind {
            SassErrorKind::Raw(string, span) => (string, span),
            e => unreachable!("unable to get raw of {:?}", e),
        }
    }

    pub(crate) const fn from_loc(message: String, loc: SpanLoc, unicode: bool) -> Self {
        SassError {
            kind: SassErrorKind::ParseError {
                message,
                loc,
                unicode,
            },
        }
    }
}

#[non_exhaustive]
#[derive(Debug, Clone)]
pub enum PublicSassErrorKind {
    ParseError {
        /// The message related to this parse error.
        ///
        /// Error messages should only be used to assist in debugging for the
        /// end user. They may change significantly between bugfix versions and
        /// should not be relied on to remain stable.
        ///
        /// Error messages do not contain the `Error: ` prefix or pretty-printed
        /// span and context information as is shown in the `Display` implementation.
        message: String,
        loc: SpanLoc,

        /// Whether or not the user allows unicode characters to be emitted in
        /// error messages.
        ///
        /// This is configurable with [`crate::Options::unicode_error_messages`]
        unicode: bool,
    },

    /// Sass was unable to find the entry-point file.
    ///
    /// Files that cannot be found using `@import`, `@use`, and `@forward` will
    /// emit [`Self::ParseError`]s
    IoError(Arc<io::Error>),

    /// The entry-point file or an imported file was not valid UTF-8.
    FromUtf8Error(String),
}

#[derive(Debug, Clone)]
enum SassErrorKind {
    /// A raw error with no additional metadata
    /// It contains only a `String` message and
    /// a span
    Raw(String, Span),
    ParseError {
        message: String,
        loc: SpanLoc,
        unicode: bool,
    },
    // we put `IoError`s in an `Arc` to allow them to be cloneable
    IoError(Arc<io::Error>),
    FromUtf8Error(String),
}

impl Display for SassError {
    // TODO: trim whitespace from start of line shown in error
    // TODO: color errors
    // TODO: integrate with codemap-diagnostics
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (message, loc, unicode) = match &self.kind {
            SassErrorKind::ParseError {
                message,
                loc,
                unicode,
            } => (message, loc, *unicode),
            SassErrorKind::FromUtf8Error(..) => return writeln!(f, "Error: Invalid UTF-8."),
            SassErrorKind::IoError(s) => return writeln!(f, "Error: {}", s),
            SassErrorKind::Raw(..) => unreachable!(),
        };

        let first_bar = if unicode { '╷' } else { ',' };
        let second_bar = if unicode { '│' } else { '|' };
        let third_bar = if unicode { '│' } else { '|' };
        let fourth_bar = if unicode { '╵' } else { '\'' };

        let line = loc.begin.line + 1;
        let col = loc.begin.column + 1;
        writeln!(f, "Error: {}", message)?;
        let padding = vec![' '; format!("{}", line).len() + 1]
            .iter()
            .collect::<String>();
        writeln!(f, "{}{}", padding, first_bar)?;
        writeln!(
            f,
            "{} {} {}",
            line,
            second_bar,
            loc.file.source_line(loc.begin.line)
        )?;
        writeln!(
            f,
            "{}{} {}{}",
            padding,
            third_bar,
            vec![' '; loc.begin.column].iter().collect::<String>(),
            vec!['^'; loc.end.column.max(loc.begin.column) - loc.begin.column.min(loc.end.column)]
                .iter()
                .collect::<String>()
        )?;
        writeln!(f, "{}{}", padding, fourth_bar)?;

        if unicode {
            writeln!(f, "./{}:{}:{}", loc.file.name(), line, col)?;
        } else {
            writeln!(f, "  {} {}:{}  root stylesheet", loc.file.name(), line, col)?;
        }
        Ok(())
    }
}

impl From<io::Error> for Box<SassError> {
    #[inline]
    fn from(error: io::Error) -> Box<SassError> {
        Box::new(SassError {
            kind: SassErrorKind::IoError(Arc::new(error)),
        })
    }
}

impl From<FromUtf8Error> for Box<SassError> {
    #[inline]
    fn from(error: FromUtf8Error) -> Box<SassError> {
        Box::new(SassError {
            kind: SassErrorKind::FromUtf8Error(format!(
                "Invalid UTF-8 character \"\\x{:X?}\"",
                error.as_bytes()[0]
            )),
        })
    }
}

impl From<(&str, Span)> for Box<SassError> {
    #[inline]
    fn from(error: (&str, Span)) -> Box<SassError> {
        Box::new(SassError {
            kind: SassErrorKind::Raw(error.0.to_owned(), error.1),
        })
    }
}

impl From<(String, Span)> for Box<SassError> {
    #[inline]
    fn from(error: (String, Span)) -> Box<SassError> {
        Box::new(SassError {
            kind: SassErrorKind::Raw(error.0, error.1),
        })
    }
}

impl Error for SassError {
    #[inline]
    fn description(&self) -> &'static str {
        "Sass parsing error"
    }
}
