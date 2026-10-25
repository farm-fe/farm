use std::iter::Peekable;

use crate::builtin::builtin_imports::*;

use crate::builtin::{
    map::{map_get, map_has_key, map_keys, map_merge, map_remove, map_set, map_values},
    modules::Module,
};

fn deep_merge_impl(map1: SassMap, map2: SassMap) -> SassMap {
    if map1.is_empty() {
        return map2;
    }
    if map2.is_empty() {
        return map1;
    }

    let mut result = map1;

    for (key, value) in map2 {
        match result.get_ref(&key.node).and_then(Value::try_map) {
            Some(result_map) => match value.try_map() {
                Some(value_map) => {
                    let merged = deep_merge_impl(result_map, value_map);
                    result.insert(key, Value::Map(merged));
                }
                None => {
                    result.insert(key, value);
                }
            },
            None => {
                result.insert(key, value);
            }
        }
    }

    result
}

fn deep_merge(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    args.max_args(2)?;

    let span = args.span();

    let map1 = args
        .get_err(0, "map1")?
        .assert_map_with_name("map1", span)?;

    let map2 = args
        .get_err(1, "map2")?
        .assert_map_with_name("map2", span)?;

    Ok(Value::Map(deep_merge_impl(map1, map2)))
}

fn deep_remove(mut args: ArgumentResult, _: &mut Visitor) -> SassResult<Value> {
    let span = args.span();

    let map = args.get_err(0, "map")?.assert_map_with_name("map", span)?;

    let key = args.get_err(1, "key")?;
    let keys = args.get_variadic()?.into_iter().map(|arg| arg.node);
    let mut keys = std::iter::once(key).chain(keys).collect::<Vec<_>>();

    let last = keys.pop();

    let map = modify_map(
        map,
        keys.into_iter(),
        |value| {
            let last = match last.as_ref() {
                Some(v) => v,
                None => return value,
            };

            match value.try_map() {
                Some(mut nested_map) if nested_map.contains(last) => {
                    nested_map.remove(last);
                    Value::Map(nested_map)
                }
                Some(..) | None => value,
            }
        },
        false,
        span,
    );

    Ok(map)
}

fn modify_map(
    map: SassMap,
    keys: impl Iterator<Item = Value>,
    modify: impl Fn(Value) -> Value,
    // default=true
    add_nesting: bool,
    span: Span,
) -> Value {
    let mut keys = keys.peekable();
    fn modify_nested_map(
        mut mutable_map: SassMap,
        mut keys: Peekable<impl Iterator<Item = Value>>,
        add_nesting: bool,
        span: Span,
        modify: impl Fn(Value) -> Value,
    ) -> SassMap {
        let key = keys.next().unwrap();

        if keys.peek().is_none() {
            let value = modify(mutable_map.get_ref(&key).cloned().unwrap_or(Value::Null));
            mutable_map.insert(key.span(span), value);
            return mutable_map;
        }

        let nested_map = mutable_map.get_ref(&key).and_then(|v| v.try_map());

        if nested_map.is_none() && !add_nesting {
            return mutable_map;
        }

        mutable_map.insert(
            key.span(span),
            Value::Map(modify_nested_map(
                nested_map.unwrap_or_default(),
                keys,
                add_nesting,
                span,
                modify,
            )),
        );

        mutable_map
    }

    if keys.peek().is_some() {
        Value::Map(modify_nested_map(map, keys, add_nesting, span, modify))
    } else {
        modify(Value::Map(map))
    }
}

pub(crate) fn declare(f: &mut Module) {
    f.insert_builtin("get", map_get);
    f.insert_builtin("has-key", map_has_key);
    f.insert_builtin("keys", map_keys);
    f.insert_builtin("merge", map_merge);
    f.insert_builtin("remove", map_remove);
    f.insert_builtin("values", map_values);
    f.insert_builtin("set", map_set);
    f.insert_builtin("deep-merge", deep_merge);
    f.insert_builtin("deep-remove", deep_remove);
}
