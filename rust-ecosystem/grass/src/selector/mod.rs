use codemap::Span;

use crate::{error::SassResult, value::Value};

pub(crate) use attribute::Attribute;
pub(crate) use common::*;
pub(crate) use complex::*;
pub(crate) use compound::*;
pub(crate) use extend::*;
pub(crate) use list::*;
pub(crate) use parse::*;
pub(crate) use simple::*;

mod attribute;
mod common;
mod complex;
mod compound;
mod extend;
mod list;
mod parse;
mod simple;

// todo: delete this selector wrapper
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selector(pub(crate) SelectorList);

impl Selector {
    /// Small wrapper around `SelectorList`'s method that turns an empty parent selector
    /// into `None`. This is a hack and in the future should be replaced.
    // todo: take Option<Self> for parent
    pub fn resolve_parent_selectors(
        &self,
        parent: &Self,
        implicit_parent: bool,
    ) -> SassResult<Self> {
        Ok(Self(self.0.clone().resolve_parent_selectors(
            if parent.is_empty() {
                None
            } else {
                Some(parent.0.clone())
            },
            implicit_parent,
        )?))
    }

    pub fn is_super_selector(&self, other: &Self) -> bool {
        self.0.is_superselector(&other.0)
    }

    pub fn contains_parent_selector(&self) -> bool {
        self.0.contains_parent_selector()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub const fn new(span: Span) -> Selector {
        Selector(SelectorList::new(span))
    }

    pub fn into_value(self) -> Value {
        self.0.to_sass_list()
    }

    pub fn unify(self, other: &Self) -> Option<Self> {
        Some(Selector(self.0.unify(&other.0)?))
    }
}
