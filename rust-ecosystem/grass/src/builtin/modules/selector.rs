use crate::{
    builtin::modules::Module,
    builtin::selector::{
        is_superselector, selector_append, selector_extend, selector_nest, selector_parse,
        selector_replace, selector_unify, simple_selectors,
    },
};

pub(crate) fn declare(f: &mut Module) {
    f.insert_builtin("is-superselector", is_superselector);
    f.insert_builtin("append", selector_append);
    f.insert_builtin("extend", selector_extend);
    f.insert_builtin("nest", selector_nest);
    f.insert_builtin("parse", selector_parse);
    f.insert_builtin("replace", selector_replace);
    f.insert_builtin("unify", selector_unify);
    f.insert_builtin("simple-selectors", simple_selectors);
}
