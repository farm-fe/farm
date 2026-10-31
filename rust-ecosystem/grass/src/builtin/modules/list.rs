use crate::builtin::builtin_imports::*;

use crate::builtin::{
    list::{append, index, is_bracketed, join, length, list_separator, nth, set_nth, zip},
    modules::Module,
};

// todo: write tests for this
fn slash(mut args: ArgumentResult, _visitor: &mut Visitor) -> SassResult<Value> {
    args.min_args(1)?;

    let span = args.span();

    let list = if args.len() == 1 {
        args.get_err(0, "elements")?.as_list()
    } else {
        args.get_variadic()?
            .into_iter()
            .map(|arg| arg.node)
            .collect()
    };

    if list.len() < 2 {
        return Err(("At least two elements are required.", span).into());
    }

    Ok(Value::List(list, ListSeparator::Slash, Brackets::None))
}

pub(crate) fn declare(f: &mut Module) {
    f.insert_builtin("append", append);
    f.insert_builtin("index", index);
    f.insert_builtin("is-bracketed", is_bracketed);
    f.insert_builtin("join", join);
    f.insert_builtin("length", length);
    f.insert_builtin("separator", list_separator);
    f.insert_builtin("nth", nth);
    f.insert_builtin("set-nth", set_nth);
    f.insert_builtin("zip", zip);
    f.insert_builtin("slash", slash);
}
