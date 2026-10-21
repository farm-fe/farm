use farmfe_docs_content::{ContentFormat, ParsedContent};
use std::collections::{HashMap, HashSet};

type Key = (String, u8);

struct Entry {
  source_hash: String,
  parsed: ParsedContent,
}

/// Process-local original syntax trees for repeated preparation by one plugin.
///
/// These trees precede collection labels, resolved references and resource
/// bindings. Each lookup returns an independent clone for the current plan.
/// Parsing/highlighting options are fixed by `prepare_with_cache`; a future
/// configurable option must become part of this identity before it is reused.
#[derive(Default)]
pub struct ContentCache {
  entries: HashMap<Key, Entry>,
  seen: HashSet<Key>,
}

impl ContentCache {
  pub(crate) fn begin(&mut self) {
    self.seen.clear();
  }

  pub(crate) fn parse(
    &mut self,
    path: &str,
    format: ContentFormat,
    source_hash: &str,
    parse: impl FnOnce() -> Result<ParsedContent, String>,
  ) -> Result<ParsedContent, String> {
    let format = match format {
      ContentFormat::Auto => 0,
      ContentFormat::Markdown => 1,
      ContentFormat::Mdx => 2,
    };
    let key = (path.to_owned(), format);
    self.seen.insert(key.clone());
    if let Some(entry) = self.entries.get(&key) {
      if entry.source_hash == source_hash {
        return Ok(entry.parsed.clone());
      }
    }
    let parsed = parse()?;
    self.entries.insert(
      key,
      Entry {
        source_hash: source_hash.to_owned(),
        parsed: parsed.clone(),
      },
    );
    Ok(parsed)
  }

  /// Only a successful complete discovery can evict previously parsed inputs.
  pub(crate) fn commit(&mut self) {
    self.entries.retain(|key, _| self.seen.contains(key));
    self.seen.clear();
  }
}

#[cfg(test)]
mod tests;
