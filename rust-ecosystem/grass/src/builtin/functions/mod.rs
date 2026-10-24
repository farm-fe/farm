// A reference to the parser is only necessary for some functions
#![allow(unused_variables)]

use std::{
    collections::{BTreeSet, HashMap},
    fmt,
    sync::atomic::{AtomicUsize, Ordering},
};

use once_cell::sync::Lazy;

use crate::{ast::ArgumentResult, error::SassResult, evaluate::Visitor, value::Value};

pub mod color;
pub mod list;
pub mod map;
pub mod math;
pub mod meta;
pub mod selector;
pub mod string;

// todo: maybe Identifier instead of str?
pub(crate) type GlobalFunctionMap = HashMap<&'static str, Builtin>;

static FUNCTION_COUNT: AtomicUsize = AtomicUsize::new(0);

/// A function implemented in rust that is accessible from within Sass
///
///
/// #### Usage
/// ```rust
/// use grass_compiler::{
///     sass_value::{ArgumentResult, SassNumber, Value},
///     Builtin, Options, Result as SassResult, Visitor,
/// };
///
/// // An example function that looks up the length of an array or map and adds 2 to it
/// fn length(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
///     args.max_args(1)?;
///
///     let len = args.get_err(0, "list")?.as_list().len();
///
///     Ok(Value::Dimension(SassNumber::new_unitless(len + 2)))
/// }
///
/// fn main() {
///     let options = Options::default().add_custom_fn("length", Builtin::new(length));
///     let css = grass_compiler::from_string("a { color: length([a, b]); }", &options).unwrap();
///
///     assert_eq!(css, "a {\n  color: 4;\n}\n");
/// }
/// ```
#[derive(Clone)]
pub struct Builtin(
    pub(crate) fn(ArgumentResult, &mut Visitor) -> SassResult<Value>,
    usize,
);

impl fmt::Debug for Builtin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Builtin")
            .field("id", &self.1)
            .field("fn_ptr", &(self.0 as usize))
            .finish()
    }
}

impl Builtin {
    pub fn new(body: fn(ArgumentResult, &mut Visitor) -> SassResult<Value>) -> Builtin {
        let count = FUNCTION_COUNT.fetch_add(1, Ordering::Relaxed);
        Self(body, count)
    }
}

impl PartialEq for Builtin {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for Builtin {}

pub(crate) static GLOBAL_FUNCTIONS: Lazy<GlobalFunctionMap> = Lazy::new(|| {
    let mut m = HashMap::new();
    color::declare(&mut m);
    list::declare(&mut m);
    map::declare(&mut m);
    math::declare(&mut m);
    meta::declare(&mut m);
    selector::declare(&mut m);
    string::declare(&mut m);
    m
});

pub(crate) static DISALLOWED_PLAIN_CSS_FUNCTION_NAMES: Lazy<BTreeSet<&str>> = Lazy::new(|| {
    GLOBAL_FUNCTIONS
        .keys()
        .copied()
        .filter(|&name| {
            !matches!(
                name,
                "rgb"
                    | "rgba"
                    | "hsl"
                    | "hsla"
                    | "grayscale"
                    | "invert"
                    | "alpha"
                    | "opacity"
                    | "saturate"
            )
        })
        .collect()
});
