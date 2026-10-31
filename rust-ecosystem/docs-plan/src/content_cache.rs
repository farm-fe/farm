use farmfe_docs_content::{ContentFormat, ParsedContent};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

type Key = (String, u8, String);

#[derive(Default)]
struct CacheProfile {
  lookup: Duration,
  parse_highlight: Duration,
  store: Duration,
  hits: usize,
  misses: usize,
  parse_errors: usize,
}

struct Entry {
  source_hash: String,
  parsed: Arc<ParsedContent>,
}

/// Process-local original syntax trees for repeated preparation by one plugin.
///
/// These trees precede collection labels, resolved references and resource
/// bindings. Shared lookups do not clone syntax trees. A final consumer obtains
/// an owned tree before applying collection-specific mutations.
/// Parsing/highlighting options are part of the identity, so one host may
/// prepare independent target presentations without reusing an incompatible tree.
#[derive(Default)]
pub struct ContentCache {
  entries: HashMap<Key, Entry>,
  seen: HashSet<Key>,
  transient: bool,
  profile: Option<CacheProfile>,
  #[cfg(test)]
  parse_calls: usize,
}

impl ContentCache {
  #[cfg(test)]
  pub(crate) fn parse_calls(&self) -> usize {
    self.parse_calls
  }
  /// Cold builds still reuse originals within one preparation, but do not keep
  /// an otherwise unused cache owner alive during route emission.
  pub(crate) fn transient() -> Self {
    Self {
      transient: true,
      ..Self::default()
    }
  }

  pub(crate) fn release_transient_originals(&mut self) {
    if self.transient {
      self.entries.clear();
    }
  }
  pub(crate) fn begin(&mut self) {
    self.seen.clear();
    self.profile = std::env::var("FARM_DOCS_VERIFY_PROFILE")
      .is_ok_and(|value| value == "1")
      .then(CacheProfile::default);
  }

  pub(crate) fn report_profile(&self, success: bool) {
    if let Some(p) = &self.profile {
      eprintln!(
        "[FarmDocsPrepareProfile] {}",
        serde_json::json!({
          "component": "prepare.cache", "success": success,
          "hits": p.hits, "misses": p.misses, "parseErrors": p.parse_errors,
          "lookupMilliseconds": p.lookup.as_secs_f64() * 1000.0,
          "parseAndHighlightInclusiveMilliseconds": p.parse_highlight.as_secs_f64() * 1000.0,
          "storeMilliseconds": p.store.as_secs_f64() * 1000.0,
          "retainedEntries": self.entries.len(),
          "scope": "one prepare call; parse includes markdown/MDX/highlight and cannot be split here"
        })
      );
    }
  }

  #[cfg(test)]
  pub(crate) fn parse(
    &mut self,
    path: &str,
    format: ContentFormat,
    source_hash: &str,
    parse: impl FnOnce() -> Result<ParsedContent, String>,
  ) -> Result<ParsedContent, String> {
    self.parse_with_options(path, format, source_hash, "default", parse)
  }

  #[cfg(test)]
  pub(crate) fn parse_with_options(
    &mut self,
    path: &str,
    format: ContentFormat,
    source_hash: &str,
    options_identity: &str,
    parse: impl FnOnce() -> Result<ParsedContent, String>,
  ) -> Result<ParsedContent, String> {
    self
      .shared_parse_with_options(path, format, source_hash, options_identity, parse)
      .map(Arc::unwrap_or_clone)
  }

  pub(crate) fn shared_parse_with_options(
    &mut self,
    path: &str,
    format: ContentFormat,
    source_hash: &str,
    options_identity: &str,
    parse: impl FnOnce() -> Result<ParsedContent, String>,
  ) -> Result<Arc<ParsedContent>, String> {
    let lookup_started = self.profile.as_ref().map(|_| Instant::now());
    let format = match format {
      ContentFormat::Auto => 0,
      ContentFormat::Markdown => 1,
      ContentFormat::Mdx => 2,
    };
    let key = (path.to_owned(), format, options_identity.to_owned());
    self.seen.insert(key.clone());
    if let Some(entry) = self.entries.get(&key) {
      if entry.source_hash == source_hash {
        let parsed = Arc::clone(&entry.parsed);
        if let (Some(p), Some(started)) = (&mut self.profile, lookup_started) {
          p.lookup += started.elapsed();
          p.hits += 1;
        }
        return Ok(parsed);
      }
    }
    if let (Some(p), Some(started)) = (&mut self.profile, lookup_started) {
      p.lookup += started.elapsed();
      p.misses += 1;
    }
    #[cfg(test)]
    {
      self.parse_calls += 1;
    }
    let parse_started = self.profile.as_ref().map(|_| Instant::now());
    let result = parse();
    if let (Some(p), Some(started)) = (&mut self.profile, parse_started) {
      p.parse_highlight += started.elapsed();
      p.parse_errors += usize::from(result.is_err());
    }
    let parsed = Arc::new(result?);
    let store_started = self.profile.as_ref().map(|_| Instant::now());
    self.entries.insert(
      key,
      Entry {
        source_hash: source_hash.to_owned(),
        parsed: Arc::clone(&parsed),
      },
    );
    if let (Some(p), Some(started)) = (&mut self.profile, store_started) {
      p.store += started.elapsed();
    }
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
