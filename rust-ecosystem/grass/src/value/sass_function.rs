use std::{fmt, sync::Arc};

use crate::{ast::AstFunctionDecl, builtin::Builtin, common::Identifier, evaluate::Environment};

/// A Sass function
///
/// The function name is stored in addition to the body
/// for use in the builtin function `inspect()`
#[derive(Clone, Eq, PartialEq)]
pub enum SassFunction {
    // todo: Cow<'static>?
    /// Builtin functions are those that have been implemented in Rust and are
    /// in the global scope.
    Builtin(Builtin, Identifier),

    // todo: maybe arc?
    /// User-defined functions are those that have been implemented in Sass using
    /// the @function rule.
    UserDefined(UserDefinedFunction),
    Plain {
        name: Identifier,
    },
}

#[derive(Debug, Clone)]
pub struct UserDefinedFunction {
    pub(crate) function: Arc<AstFunctionDecl>,
    pub name: Identifier,
    pub(crate) env: Environment,
}

impl PartialEq for UserDefinedFunction {
    fn eq(&self, other: &Self) -> bool {
        self.function == other.function && self.name == other.name
    }
}

impl Eq for UserDefinedFunction {}

impl SassFunction {
    /// Get the name of the function referenced
    ///
    /// Used mainly in debugging and `inspect()`
    pub fn name(&self) -> Identifier {
        match self {
            Self::Builtin(_, name)
            | Self::UserDefined(UserDefinedFunction { name, .. })
            | Self::Plain { name } => *name,
        }
    }

    /// Whether the function is builtin or user-defined
    ///
    /// Used only in `std::fmt::Debug` for `SassFunction`
    fn kind(&self) -> &'static str {
        match &self {
            Self::Plain { .. } => "Plain",
            Self::Builtin(..) => "Builtin",
            Self::UserDefined { .. } => "UserDefined",
        }
    }
}

impl fmt::Debug for SassFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SassFunction")
            .field("name", &self.name())
            .field("kind", &self.kind())
            .finish()
    }
}
