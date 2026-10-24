use std::fmt;

/// The selector namespace.
///
/// If this is `None`, this matches all elements in the default namespace. If
/// it's `Empty`, this matches all elements that aren't in any
/// namespace. If it's `Asterisk`, this matches all elements in any namespace.
/// Otherwise, it matches all elements in the given namespace.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum Namespace {
    Empty,
    Asterisk,
    Other(Box<str>),
    None,
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "|"),
            Self::Asterisk => write!(f, "*|"),
            Self::Other(namespace) => write!(f, "{}|", namespace),
            Self::None => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct QualifiedName {
    pub ident: String,
    pub namespace: Namespace,
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.namespace)?;
        f.write_str(&self.ident)
    }
}

pub(crate) struct Specificity {
    pub min: i32,
    pub max: i32,
}

impl Specificity {
    pub const fn new(min: i32, max: i32) -> Self {
        Specificity { min, max }
    }
}
