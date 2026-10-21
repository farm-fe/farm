use crate::{ConfigError, ResolvedCollection};
use std::collections::HashSet;

fn fallback<'a>(
  collection: &ResolvedCollection,
  collections: &'a [ResolvedCollection],
) -> Result<Option<&'a ResolvedCollection>, ConfigError> {
  let Some(locale) = &collection.fallback_locale else {
    return Ok(None);
  };
  let field = format!("collections.{}.fallbackLocale", collection.public.id);
  if collection.public.group.is_none() || locale == &collection.public.locale {
    return Err(ConfigError::new(
      "DOCS_CONFIG_FALLBACK",
      &field,
      "require an explicit group and a different source locale",
    ));
  }
  collections
    .iter()
    .find(|source| {
      source.public.group == collection.public.group
        && source.public.kind == collection.public.kind
        && source.public.version == collection.public.version
        && source.public.locale == *locale
    })
    .map(Some)
    .ok_or_else(|| {
      ConfigError::new(
        "DOCS_CONFIG_FALLBACK",
        &field,
        "source locale is not configured in the same group and version",
      )
    })
}

pub fn validate(collections: &[ResolvedCollection]) -> Result<(), ConfigError> {
  let mut slots = HashSet::new();
  for collection in collections {
    if let Some(group) = &collection.public.group {
      if !slots.insert((
        collection.public.kind,
        group.clone(),
        collection.public.locale.clone(),
        collection.public.version.clone(),
      )) {
        return Err(ConfigError::new(
          "DOCS_CONFIG_GROUP",
          &format!("collections.{}.group", collection.public.id),
          "duplicate locale/version slot",
        ));
      }
    }
    let mut seen = HashSet::new();
    let mut current = collection;
    loop {
      if !seen.insert(current.public.id.as_str()) {
        return Err(ConfigError::new(
          "DOCS_CONFIG_FALLBACK",
          &format!("collections.{}.fallbackLocale", collection.public.id),
          "fallback cycle",
        ));
      }
      let Some(source) = fallback(current, collections)? else {
        break;
      };
      current = source;
    }
  }
  Ok(())
}
