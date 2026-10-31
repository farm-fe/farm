use crate::builtin::builtin_imports::*;

/// map.get($map, $key, $keys...)
/// map-get($map, $key, $keys...)
///
/// If $keys is empty, returns the value in $map associated with $key.
/// If $map doesn’t have a value associated with $key, returns null.
/// If $keys is not empty, follows the set of keys including $key and
/// excluding the last key in $keys, from left to right, to find the
/// nested map targeted for searching.
/// Returns the value in the targeted map associated with the last key
/// in $keys.
/// Returns null if the map does not have a value associated with the
/// key, or if any key in $keys is missing from a map or references a
/// value that is not a map.
///
/// https://sass-lang.com/documentation/modules/map/
pub(crate) fn map_get(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let key = args.get_err(1, "key")?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;

    // since we already extracted the map and first key,
    // neither will be returned in the variadic args list
    let keys = args.get_variadic()?;

    let mut val = map.get(&key).unwrap_or(Value::Null);
    for key in keys {
        // if at any point we find a value that's not a map,
        // we return null
        let val_map = match val.try_map() {
            Some(val_map) => val_map,
            None => return Ok(Value::Null),
        };

        val = val_map.get(&key).unwrap_or(Value::Null);
    }

    Ok(val)
}

pub(crate) fn map_has_key(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let key = args.get_err(1, "key")?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;

    // since we already extracted the map and first key,
    // neither will be returned in the variadic args list
    let keys = args.get_variadic()?;

    let mut val = match map.get(&key) {
        Some(v) => v,
        None => return Ok(Value::False),
    };
    for key in keys {
        // if at any point we find a value that's not a map,
        // we return null
        let val_map = match val.try_map() {
            Some(val_map) => val_map,
            None => return Ok(Value::False),
        };

        val = match val_map.get(&key) {
            Some(v) => v,
            None => return Ok(Value::False),
        };
    }

    Ok(Value::True)
}

pub(crate) fn map_keys(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;
    Ok(Value::List(
        map.keys(),
        ListSeparator::Comma,
        Brackets::None,
    ))
}

pub(crate) fn map_values(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    args.max_args(1)?;
    let map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;
    Ok(Value::List(
        map.values(),
        ListSeparator::Comma,
        Brackets::None,
    ))
}

pub(crate) fn map_merge(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    if args.len() == 1 {
        return Err(("Expected $args to contain a key.", args.span()).into());
    }

    let map2_position = args.len().saturating_sub(1);

    let mut map1 = args
        .get_err(0, "map1")?
        .assert_map_with_name("map1", args.span())?;

    let map2 = args
        .get_err(map2_position, "map2")?
        .assert_map_with_name("map2", args.span())?;

    let keys = args.get_variadic()?;

    if keys.is_empty() {
        map1.merge(map2);
    } else {
        let mut current_map = map1.clone();
        let mut map_queue = Vec::new();

        for key in keys {
            match current_map.get(&key) {
                Some(Value::Map(m1)) => {
                    current_map = m1.clone();
                    map_queue.push((key, m1));
                }
                Some(..) | None => {
                    current_map = SassMap::new();
                    map_queue.push((key, SassMap::new()));
                }
            }
        }

        match map_queue.last_mut() {
            Some((_, m)) => {
                m.merge(map2);
            }
            None => unreachable!(),
        };

        while let Some((key, queued_map)) = map_queue.pop() {
            match map_queue.last_mut() {
                Some((_, map)) => {
                    map.insert(key, Value::Map(queued_map));
                }
                None => {
                    map1.insert(key, Value::Map(queued_map));
                    break;
                }
            }
        }
    }

    Ok(Value::Map(map1))
}

pub(crate) fn map_remove(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let mut map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;
    let keys = args.get_variadic()?;
    for key in keys {
        map.remove(&key);
    }
    Ok(Value::Map(map))
}

pub(crate) fn map_set(mut args: ArgumentResult, visitor: &mut Visitor) -> SassResult<Value> {
    let key_position = args.len().saturating_sub(2);
    let value_position = args.len().saturating_sub(1);

    let mut map = args
        .get_err(0, "map")?
        .assert_map_with_name("map", args.span())?;

    let key = Spanned {
        node: args.get_err(key_position, "key")?,
        span: args.span(),
    };
    let value = args.get_err(value_position, "value")?;

    let keys = args.get_variadic()?;

    if keys.is_empty() {
        map.insert(key, value);
    } else {
        let mut current_map = map.clone();
        let mut map_queue = Vec::new();

        for key in keys {
            match current_map.get(&key) {
                Some(Value::Map(m1)) => {
                    current_map = m1.clone();
                    map_queue.push((key, m1));
                }
                Some(..) | None => {
                    current_map = SassMap::new();
                    map_queue.push((key, SassMap::new()));
                }
            }
        }

        match map_queue.last_mut() {
            Some((_, m)) => m.insert(key, value),
            None => unreachable!(),
        };

        while let Some((key, queued_map)) = map_queue.pop() {
            match map_queue.last_mut() {
                Some((_, next_map)) => {
                    next_map.insert(key, Value::Map(queued_map));
                }
                None => {
                    map.insert(key, Value::Map(queued_map));
                    break;
                }
            }
        }
    }

    Ok(Value::Map(map))
}

pub(crate) fn declare(f: &mut GlobalFunctionMap) {
    f.insert("map-get", Builtin::new(map_get));
    f.insert("map-has-key", Builtin::new(map_has_key));
    f.insert("map-keys", Builtin::new(map_keys));
    f.insert("map-values", Builtin::new(map_values));
    f.insert("map-merge", Builtin::new(map_merge));
    f.insert("map-remove", Builtin::new(map_remove));
}
