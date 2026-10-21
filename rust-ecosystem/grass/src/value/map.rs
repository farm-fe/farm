use std::{slice::Iter, vec::IntoIter};

use codemap::Spanned;

use crate::{
    common::{Brackets, ListSeparator},
    value::Value,
};

#[derive(Debug, Clone, Default)]
pub struct SassMap(Vec<(Spanned<Value>, Value)>);

impl PartialEq for SassMap {
    fn eq(&self, other: &Self) -> bool {
        if self.0.len() != other.0.len() {
            return false;
        }
        for (key, value) in &self.0 {
            if !other
                .0
                .iter()
                .any(|(key2, value2)| key.node == key2.node && value == value2)
            {
                return false;
            }
        }
        true
    }
}

impl Eq for SassMap {}

impl SassMap {
    pub const fn new() -> SassMap {
        SassMap(Vec::new())
    }

    pub const fn new_with(elements: Vec<(Spanned<Value>, Value)>) -> SassMap {
        SassMap(elements)
    }

    pub fn get(self, key: &Value) -> Option<Value> {
        for (k, v) in self.0 {
            if &k.node == key {
                return Some(v);
            }
        }

        None
    }

    pub fn get_ref(&self, key: &Value) -> Option<&Value> {
        for (k, v) in &self.0 {
            if &k.node == key {
                return Some(v);
            }
        }

        None
    }

    pub fn remove(&mut self, key: &Value) {
        self.0.retain(|(ref k, ..)| k.not_equals(key));
    }

    pub fn merge(&mut self, other: SassMap) {
        for (key, value) in other {
            self.insert(key, value);
        }
    }

    pub fn iter(&self) -> Iter<(Spanned<Value>, Value)> {
        self.0.iter()
    }

    pub fn keys(self) -> Vec<Value> {
        self.0.into_iter().map(|(k, ..)| k.node).collect()
    }

    pub fn values(self) -> Vec<Value> {
        self.0.into_iter().map(|(.., v)| v).collect()
    }

    pub fn contains(&self, key: &Value) -> bool {
        self.0.iter().any(|(k, ..)| &k.node == key)
    }

    pub fn as_list(self) -> Vec<Value> {
        self.0
            .into_iter()
            .map(|(k, v)| Value::List(vec![k.node, v], ListSeparator::Space, Brackets::None))
            .collect()
    }

    /// Returns true if the key already exists
    pub fn insert(&mut self, key: Spanned<Value>, value: Value) -> bool {
        for (ref k, ref mut v) in &mut self.0 {
            if k.node == key.node {
                *v = value;
                return true;
            }
        }
        self.0.push((key, value));
        false
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl IntoIterator for SassMap {
    type Item = (Spanned<Value>, Value);
    type IntoIter = IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
