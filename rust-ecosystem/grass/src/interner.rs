use lasso::{Rodeo, Spur};

use std::cell::RefCell;
use std::fmt::{self, Display};

thread_local!(static STRINGS: RefCell<Rodeo<Spur>> = RefCell::new(Rodeo::default()));

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct InternedString(Spur);

impl InternedString {
    pub fn get_or_intern<T: AsRef<str>>(s: T) -> Self {
        Self(STRINGS.with(|interner| interner.borrow_mut().get_or_intern(s)))
    }

    #[allow(dead_code)]
    pub fn resolve(self) -> String {
        STRINGS.with(|interner| interner.borrow().resolve(&self.0).to_owned())
    }

    #[allow(dead_code)]
    pub fn is_empty(self) -> bool {
        self.resolve_ref() == ""
    }

    // todo: no need for unsafe here
    pub fn resolve_ref<'a>(self) -> &'a str {
        unsafe { STRINGS.with(|interner| interner.as_ptr().as_ref().unwrap().resolve(&self.0)) }
    }
}

impl Display for InternedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        STRINGS.with(|interner| write!(f, "{}", interner.borrow().resolve(&self.0)))
    }
}
