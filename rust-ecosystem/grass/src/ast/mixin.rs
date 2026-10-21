use std::fmt;

use crate::{
    ast::ArgumentResult,
    error::SassResult,
    evaluate::{Environment, Visitor},
};

pub(crate) type BuiltinMixin = fn(ArgumentResult, &mut Visitor) -> SassResult<()>;

pub(crate) use crate::ast::AstMixin as UserDefinedMixin;

#[derive(Clone)]
pub(crate) enum Mixin {
    UserDefined(UserDefinedMixin, Environment),
    Builtin(BuiltinMixin),
}

impl fmt::Debug for Mixin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UserDefined(u, ..) => f
                .debug_struct("AstMixin")
                .field("name", &u.name)
                .field("args", &u.args)
                .field("body", &u.body)
                .field("has_content", &u.has_content)
                .finish(),
            Self::Builtin(..) => f.debug_struct("BuiltinMixin").finish(),
        }
    }
}
