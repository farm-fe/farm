use std::{cell::Cell, collections::BTreeMap, rc::Rc};

use crate::common::{Identifier, ListSeparator};

use super::Value;

#[derive(Debug, Clone)]
pub struct ArgList {
    pub elems: Vec<Value>,
    were_keywords_accessed: Rc<Cell<bool>>,
    // todo: special wrapper around this field to avoid having to make it private?
    keywords: BTreeMap<Identifier, Value>,
    pub separator: ListSeparator,
}

impl PartialEq for ArgList {
    fn eq(&self, other: &Self) -> bool {
        self.elems == other.elems
            && self.keywords == other.keywords
            && self.separator == other.separator
    }
}

impl Eq for ArgList {}

impl ArgList {
    pub fn new(
        elems: Vec<Value>,
        were_keywords_accessed: Rc<Cell<bool>>,
        keywords: BTreeMap<Identifier, Value>,
        separator: ListSeparator,
    ) -> Self {
        debug_assert!(
            !(*were_keywords_accessed).get(),
            "expected args to initialize with unaccessed keywords"
        );

        Self {
            elems,
            were_keywords_accessed,
            keywords,
            separator,
        }
    }

    pub fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_blank(&self) -> bool {
        !self.is_empty() && (self.elems.iter().all(Value::is_blank))
    }

    pub fn keywords(&self) -> &BTreeMap<Identifier, Value> {
        (*self.were_keywords_accessed).set(true);
        &self.keywords
    }

    pub fn into_keywords(self) -> BTreeMap<Identifier, Value> {
        (*self.were_keywords_accessed).set(true);
        self.keywords
    }
}
