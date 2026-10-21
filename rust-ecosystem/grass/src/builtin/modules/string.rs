use crate::builtin::{
    modules::Module,
    string::{
        quote, str_index, str_insert, str_length, str_slice, str_split, to_lower_case,
        to_upper_case, unquote,
    },
};

#[cfg(feature = "random")]
use crate::builtin::string::unique_id;

pub(crate) fn declare(f: &mut Module) {
    f.insert_builtin("quote", quote);
    f.insert_builtin("index", str_index);
    f.insert_builtin("insert", str_insert);
    f.insert_builtin("length", str_length);
    f.insert_builtin("slice", str_slice);
    f.insert_builtin("split", str_split);
    f.insert_builtin("to-lower-case", to_lower_case);
    f.insert_builtin("to-upper-case", to_upper_case);
    #[cfg(feature = "random")]
    f.insert_builtin("unique-id", unique_id);
    f.insert_builtin("unquote", unquote);
}
