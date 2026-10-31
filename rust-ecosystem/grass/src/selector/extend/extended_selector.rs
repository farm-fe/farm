use std::{
    cell::RefCell,
    collections::{hash_set::IntoIter, HashSet},
    hash::{Hash, Hasher},
    ops::Deref,
    ptr,
    rc::Rc,
};

use crate::selector::{Selector, SelectorList};

#[derive(Debug, Clone)]
pub(crate) struct ExtendedSelector(Rc<RefCell<SelectorList>>);

impl PartialEq for ExtendedSelector {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Eq for ExtendedSelector {}

impl Hash for ExtendedSelector {
    // We hash the ptr here for efficiency.
    // TODO: is this an issue? it probably is,
    // but I haven't managed to find a test case
    // that exhibits it.
    fn hash<H: Hasher>(&self, state: &mut H) {
        ptr::hash(&*self.0, state);
        // in case we need to hash the actual value:
        // self.0.borrow().hash(state);
    }
}

impl ExtendedSelector {
    pub fn new(selector: SelectorList) -> Self {
        Self(Rc::new(RefCell::new(selector)))
    }

    pub fn is_invisible(&self) -> bool {
        (*self.0).borrow().is_invisible()
    }

    pub fn into_selector(self) -> Selector {
        Selector(match Rc::try_unwrap(self.0) {
            Ok(v) => v.into_inner(),
            Err(v) => v.borrow().clone(),
        })
    }

    pub fn as_selector_list(&self) -> impl Deref<Target = SelectorList> + '_ {
        self.0.borrow()
    }

    pub fn set_inner(&mut self, selector: SelectorList) {
        self.0.replace(selector);
    }
}

/// There is the potential for danger here by modifying the hash
/// through `RefCell`, but I haven't come up with a good solution
/// for this yet (we can't just use a `Vec` because linear insert)
/// is too big of a penalty
///
/// In practice, I have yet to find a test case that can demonstrate
/// an issue with storing a `RefCell`.
#[derive(Clone, Debug)]
pub(crate) struct SelectorHashSet(HashSet<ExtendedSelector>);

impl SelectorHashSet {
    pub fn new() -> Self {
        Self(HashSet::new())
    }

    pub fn insert(&mut self, selector: ExtendedSelector) {
        self.0.insert(selector);
    }
}

impl IntoIterator for SelectorHashSet {
    type Item = ExtendedSelector;
    type IntoIter = IntoIter<ExtendedSelector>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
