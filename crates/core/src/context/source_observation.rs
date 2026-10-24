//! Evidence of inputs consumed through controlled native-plugin reads.
//!
//! The ledger is scoped to one compiler generation. Recording a new input is
//! not itself an invalidation: contradictory observations or changed inputs
//! are errors. It does not claim to intercept arbitrary plugin filesystem IO.
use std::{
  collections::{BTreeMap, BTreeSet},
  fs, io,
  path::{Path, PathBuf},
  sync::atomic::{AtomicU64, Ordering},
};

use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

fn digest(bytes: &[u8]) -> String {
  format!("{:x}", Sha256::digest(bytes))
}

fn text(path: &Path) -> io::Result<String> {
  let value = path
    .to_str()
    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "observed path must be UTF-8"))?
    .replace('\\', "/");
  if let Some(unc) = value.strip_prefix("//?/UNC/") {
    return Ok(format!("//{unc}"));
  }
  Ok(value.strip_prefix("//?/").unwrap_or(&value).to_owned())
}

fn query_text(path: &Path) -> io::Result<String> {
  text(&std::path::absolute(path)?)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", deny_unknown_fields)]
pub enum ObservedValue {
  File {
    canonical: String,
    hash: String,
  },
  /// An exists/is_file/is_dir query, including negative lookup results.
  Metadata {
    canonical: Option<String>,
    file_type: String,
  },
  /// An ordered directory listing returned to the reader, not a recursive glob.
  Directory {
    canonical: String,
    entries: Vec<(String, String)>,
  },
  /// Only a digest is retained. Values are never included in diagnostics.
  Environment {
    present: bool,
    hash: String,
  },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct InputObservation {
  pub reader: String,
  pub query: String,
  pub value: ObservedValue,
}

#[derive(Default)]
struct State {
  observations: BTreeMap<(String, String, &'static str), InputObservation>,
  derived: BTreeMap<String, BTreeSet<String>>,
  failure: Option<String>,
  file_hashes: BTreeMap<String, String>,
  facts: BTreeMap<(String, &'static str), ObservedValue>,
  revision: u64,
  frozen: bool,
}

/// Append-only facts for one compilation attempt. This is not a snapshot seal.
#[derive(Default)]
pub struct SourceObservationLedger {
  generation: AtomicU64,
  state: Mutex<State>,
}

impl SourceObservationLedger {
  /// Verify persisted observations without claiming that this generation read them.
  /// Callers must validate the trust/scope of paths before restoring any cache.
  pub fn verify_records(records: &[InputObservation]) -> io::Result<()> {
    let ledger = Self::default();
    ledger.insert_records(records)?;
    ledger.verify()
  }

  /// Restore a validated cache's input proof; a changed observation is an error.
  /// This does not restore derived-module relationships or attest new transforms.
  pub fn restore_verified(&self, records: &[InputObservation]) -> io::Result<()> {
    Self::verify_records(records)?;
    self.insert_records(records)
  }

  fn insert_records(&self, records: &[InputObservation]) -> io::Result<()> {
    let generation = self.generation();
    for record in records {
      let operation = match &record.value {
        ObservedValue::File { .. } => "file",
        ObservedValue::Metadata { .. } => "metadata",
        ObservedValue::Directory { .. } => "directory",
        ObservedValue::Environment { .. } => "environment",
      };
      self.insert(
        generation,
        &record.reader,
        record.query.clone(),
        operation,
        record.value.clone(),
      )?;
    }
    Ok(())
  }
  pub fn generation(&self) -> u64 {
    self.generation.load(Ordering::SeqCst)
  }

  /// Export only after successful compiler sealing; this does not repeat filesystem IO.
  pub fn sealed_records(&self) -> io::Result<Vec<InputObservation>> {
    let state = self.state.lock();
    if !state.frozen || state.failure.is_some() {
      return Err(io::Error::other(
        "[FARM_INPUT_OBSERVATION] generation is not successfully sealed",
      ));
    }
    Ok(state.observations.values().cloned().collect())
  }

  fn failed(&self, message: &'static str) -> io::Error {
    self.state.lock().failure = Some(message.into());
    io::Error::other(message)
  }

  fn absent(
    &self,
    generation: u64,
    reader: &str,
    path: &Path,
    operation: &'static str,
  ) -> io::Result<()> {
    self.insert(
      generation,
      reader,
      query_text(path)?,
      operation,
      ObservedValue::Metadata {
        canonical: None,
        file_type: "absent".into(),
      },
    )
  }
  /// Call only after the previous generation's work has quiesced.
  pub fn begin_generation(&self) -> u64 {
    let mut state = self.state.lock();
    *state = State::default();
    self.generation.fetch_add(1, Ordering::SeqCst) + 1
  }

  fn insert(
    &self,
    generation: u64,
    reader: &str,
    query: String,
    operation: &'static str,
    value: ObservedValue,
  ) -> io::Result<()> {
    let mut state = self.state.lock();
    if generation != self.generation.load(Ordering::SeqCst) {
      return Err(io::Error::other(
        "[FARM_INPUT_OBSERVATION] generation changed during read",
      ));
    }
    if state.frozen {
      state.failure = Some("[FARM_INPUT_OBSERVATION] read after generation was sealed".into());
      return Err(io::Error::other(
        "[FARM_INPUT_OBSERVATION] read after generation was sealed",
      ));
    }
    let fact_key = (query.clone(), operation);
    let contradictory = state
      .facts
      .get(&fact_key)
      .is_some_and(|previous| previous != &value)
      || matches!(&value, ObservedValue::File { canonical, hash }
        if state.file_hashes.get(canonical).is_some_and(|previous| previous != hash));
    if contradictory {
      state.failure = Some("[FARM_INPUT_OBSERVATION] contradictory input across readers".into());
      return Err(io::Error::other(
        "[FARM_INPUT_OBSERVATION] contradictory input across readers",
      ));
    }
    let key = (reader.to_owned(), query.clone(), operation);
    let observation = InputObservation {
      reader: reader.to_owned(),
      query,
      value,
    };
    if state
      .observations
      .get(&key)
      .is_some_and(|previous| previous != &observation)
    {
      let message = "[FARM_INPUT_OBSERVATION] input changed within a compilation generation";
      state.failure = Some(message.into());
      return Err(io::Error::other(message));
    }
    if state.observations.get(&key) == Some(&observation) {
      return Ok(());
    }
    if let ObservedValue::File { canonical, hash } = &observation.value {
      state.file_hashes.insert(canonical.clone(), hash.clone());
    }
    state.facts.insert(fact_key, observation.value.clone());
    state.observations.insert(key, observation);
    state.revision += 1;
    Ok(())
  }

  /// Read and record the exact bytes that are returned to the caller.
  pub fn read_file(&self, reader: &str, path: &Path) -> io::Result<Vec<u8>> {
    let generation = self.generation.load(Ordering::SeqCst);
    let before = match fs::canonicalize(path) {
      Ok(path) => path,
      Err(error) if error.kind() == io::ErrorKind::NotFound => {
        self.absent(generation, reader, path, "file")?;
        return Err(error);
      }
      Err(_) => return Err(self.failed("[FARM_INPUT_OBSERVATION] file identity lookup failed")),
    };
    let bytes = fs::read(path)
      .map_err(|_| self.failed("[FARM_INPUT_OBSERVATION] controlled file read failed"))?;
    let after = fs::canonicalize(path)
      .map_err(|_| self.failed("[FARM_INPUT_OBSERVATION] file disappeared during read"))?;
    if before != after {
      return Err(self.failed("[FARM_INPUT_OBSERVATION] file identity changed during read"));
    }
    self.insert(
      generation,
      reader,
      query_text(path)?,
      "file",
      ObservedValue::File {
        canonical: text(&after)?,
        hash: digest(&bytes),
      },
    )?;
    Ok(bytes)
  }

  /// Query metadata without confusing permission errors with a missing path.
  pub fn metadata(&self, reader: &str, path: &Path) -> io::Result<Option<fs::Metadata>> {
    let generation = self.generation.load(Ordering::SeqCst);
    let metadata = match fs::metadata(path) {
      Ok(metadata) => Some(metadata),
      Err(error) if error.kind() == io::ErrorKind::NotFound => None,
      Err(_) => return Err(self.failed("[FARM_INPUT_OBSERVATION] metadata query failed")),
    };
    let (canonical, file_type) = match metadata.as_ref() {
      Some(metadata) => (
        Some(text(&fs::canonicalize(path)?)?),
        if metadata.is_file() {
          "file"
        } else if metadata.is_dir() {
          "directory"
        } else {
          "other"
        },
      ),
      None => (None, "absent"),
    };
    self.insert(
      generation,
      reader,
      query_text(path)?,
      "metadata",
      ObservedValue::Metadata {
        canonical,
        file_type: file_type.into(),
      },
    )?;
    Ok(metadata)
  }

  /// Return a deterministic listing; callers must consume this same ordering.
  pub fn read_directory(&self, reader: &str, path: &Path) -> io::Result<Vec<PathBuf>> {
    let generation = self.generation.load(Ordering::SeqCst);
    let canonical = match fs::canonicalize(path) {
      Ok(path) => text(&path)?,
      Err(error) if error.kind() == io::ErrorKind::NotFound => {
        self.absent(generation, reader, path, "directory")?;
        return Err(error);
      }
      Err(_) => {
        return Err(self.failed("[FARM_INPUT_OBSERVATION] directory identity lookup failed"));
      }
    };
    let mut entries = Vec::new();
    for entry in fs::read_dir(path)
      .map_err(|_| self.failed("[FARM_INPUT_OBSERVATION] directory read failed"))?
    {
      let entry =
        entry.map_err(|_| self.failed("[FARM_INPUT_OBSERVATION] directory entry read failed"))?;
      let kind = entry
        .file_type()
        .map_err(|_| self.failed("[FARM_INPUT_OBSERVATION] directory entry type failed"))?;
      entries.push((
        entry.path(),
        if kind.is_symlink() {
          "symlink"
        } else if kind.is_dir() {
          "directory"
        } else if kind.is_file() {
          "file"
        } else {
          "other"
        },
      ));
    }
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    let rows = entries
      .iter()
      .map(|(path, kind)| Ok((query_text(path)?, (*kind).into())))
      .collect::<io::Result<Vec<_>>>()?;
    self.insert(
      generation,
      reader,
      query_text(path)?,
      "directory",
      ObservedValue::Directory {
        canonical,
        entries: rows,
      },
    )?;
    Ok(entries.into_iter().map(|(path, _)| path).collect())
  }

  /// Read an environment value without retaining its plaintext in the ledger.
  pub fn read_environment(&self, reader: &str, name: &str) -> io::Result<Option<String>> {
    let generation = self.generation.load(Ordering::SeqCst);
    let value = match std::env::var(name) {
      Ok(value) => Some(value),
      Err(std::env::VarError::NotPresent) => None,
      Err(_) => {
        return Err(self.failed("[FARM_INPUT_OBSERVATION] environment value is not valid UTF-8"));
      }
    };
    self.insert(
      generation,
      reader,
      name.into(),
      "environment",
      ObservedValue::Environment {
        present: value.is_some(),
        hash: digest(value.as_deref().unwrap_or_default().as_bytes()),
      },
    )?;
    Ok(value)
  }

  /// Record consumption of another module's already-observed derived result.
  /// Call before consuming it. This is not permission to attest arbitrary IO.
  pub fn consume_derived(&self, generation: u64, reader: &str, producer: &str) -> io::Result<()> {
    let mut state = self.state.lock();
    if generation != self.generation.load(Ordering::SeqCst) || state.frozen {
      return Err(io::Error::other(
        "[FARM_INPUT_OBSERVATION] stale derived input generation",
      ));
    }
    if state
      .derived
      .entry(reader.into())
      .or_default()
      .insert(producer.into())
    {
      state.revision += 1;
    }
    Ok(())
  }

  pub fn producers(&self, reader: &str) -> Vec<String> {
    self
      .state
      .lock()
      .derived
      .get(reader)
      .map(|items| items.iter().cloned().collect())
      .unwrap_or_default()
  }

  /// Return a positive read proof only; negative facts are verified separately.
  pub fn file_read_hash(&self, reader: &str, canonical: &str) -> Option<String> {
    let state = self.state.lock();
    if state.failure.is_some() {
      return None;
    }
    state.observations.values().find_map(|record| {
      if record.reader != reader {
        return None;
      }
      match &record.value {
        ObservedValue::File {
          canonical: observed,
          hash,
        } if observed == canonical => Some(hash.clone()),
        _ => None,
      }
    })
  }

  pub fn records(&self) -> io::Result<Vec<InputObservation>> {
    let state = self.state.lock();
    if let Some(failure) = &state.failure {
      return Err(io::Error::other(failure.clone()));
    }
    Ok(state.observations.values().cloned().collect())
  }

  /// Recheck every positive and negative fact. No metadata-only file shortcut.
  fn verify_snapshot(&self) -> io::Result<(u64, u64)> {
    let (generation, revision, observations) = {
      let state = self.state.lock();
      if let Some(failure) = &state.failure {
        return Err(io::Error::other(failure.clone()));
      }
      (
        self.generation(),
        state.revision,
        state.observations.values().cloned().collect::<Vec<_>>(),
      )
    };
    for record in observations {
      let valid = match record.value {
        ObservedValue::File { canonical, hash } => {
          fs::canonicalize(&record.query)
            .and_then(|path| text(&path))
            .ok()
            .as_ref()
            == Some(&canonical)
            && fs::read(&record.query)
              .map(|bytes| digest(&bytes) == hash)
              .unwrap_or(false)
        }
        ObservedValue::Metadata {
          canonical,
          file_type,
        } => match fs::metadata(&record.query) {
          Ok(metadata) => {
            let kind = if metadata.is_file() {
              "file"
            } else if metadata.is_dir() {
              "directory"
            } else {
              "other"
            };
            file_type == kind
              && fs::canonicalize(&record.query)
                .and_then(|path| text(&path))
                .ok()
                == canonical
          }
          Err(error) => {
            error.kind() == io::ErrorKind::NotFound && file_type == "absent" && canonical.is_none()
          }
        },
        ObservedValue::Directory { canonical, entries } => {
          let probe = Self::default();
          probe
            .read_directory(&record.reader, Path::new(&record.query))
            .is_ok()
            && probe.records()?.into_iter().any(|item| {
              item.value
                == ObservedValue::Directory {
                  canonical: canonical.clone(),
                  entries: entries.clone(),
                }
            })
        }
        ObservedValue::Environment { present, hash } => match std::env::var(&record.query) {
          Ok(value) => present && digest(value.as_bytes()) == hash,
          Err(std::env::VarError::NotPresent) => !present && digest(b"") == hash,
          Err(_) => false,
        },
      };
      if !valid {
        return Err(io::Error::other(
          "[FARM_INPUT_OBSERVATION] consumed input or query changed",
        ));
      }
    }
    let state = self.state.lock();
    if self.generation() != generation || state.revision != revision || state.failure.is_some() {
      return Err(io::Error::other(
        "[FARM_INPUT_OBSERVATION] generation changed while verifying",
      ));
    }
    Ok((generation, revision))
  }

  pub fn verify(&self) -> io::Result<()> {
    self.verify_snapshot().map(|_| ())
  }

  /// Freeze only after all compiler/plugin readers have completed.
  pub fn seal(&self) -> io::Result<()> {
    let (generation, revision) = self.verify_snapshot()?;
    let mut state = self.state.lock();
    if self.generation() != generation || state.revision != revision || state.failure.is_some() {
      return Err(io::Error::other(
        "[FARM_INPUT_OBSERVATION] generation changed before sealing",
      ));
    }
    state.frozen = true;
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::sync::atomic::{AtomicUsize, Ordering};
  static NEXT: AtomicUsize = AtomicUsize::new(0);
  fn directory() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
      "farm-observation-{}-{}",
      std::process::id(),
      NEXT.fetch_add(1, Ordering::SeqCst)
    ));
    fs::create_dir_all(&root).unwrap();
    root
  }
  #[test]
  fn new_consumed_file_is_valid_without_a_seed() {
    let root = directory();
    let path = root.join("source.txt");
    fs::write(&path, b"original").unwrap();
    let ledger = SourceObservationLedger::default();
    assert_eq!(ledger.read_file("reader", &path).unwrap(), b"original");
    ledger.verify().unwrap();
    fs::write(&path, b"changed").unwrap();
    assert!(ledger.verify().is_err());
    fs::remove_dir_all(root).unwrap();
  }
  #[test]
  fn conflicting_reads_cannot_be_replaced_with_the_last_value() {
    let root = directory();
    let path = root.join("source.txt");
    fs::write(&path, b"first").unwrap();
    let ledger = SourceObservationLedger::default();
    ledger.read_file("reader", &path).unwrap();
    fs::write(&path, b"second").unwrap();
    assert!(ledger.read_file("reader", &path).is_err());
    fs::write(&path, b"first").unwrap();
    assert!(ledger.verify().is_err());
    fs::remove_dir_all(root).unwrap();
  }
  #[test]
  fn absent_lookup_and_directory_addition_invalidate() {
    let root = directory();
    let missing = root.join("later.ts");
    let ledger = SourceObservationLedger::default();
    assert!(ledger.metadata("resolver", &missing).unwrap().is_none());
    ledger.read_directory("glob", &root).unwrap();
    ledger.verify().unwrap();
    fs::write(missing, b"new").unwrap();
    assert!(ledger.verify().is_err());
    fs::remove_dir_all(root).unwrap();
  }
  #[test]
  fn generation_reset_drops_old_facts_and_derived_links() {
    let ledger = SourceObservationLedger::default();
    ledger
      .consume_derived(ledger.generation(), "css", "script")
      .unwrap();
    assert_eq!(ledger.producers("css"), vec!["script"]);
    assert_eq!(ledger.begin_generation(), 1);
    assert!(ledger.producers("css").is_empty());
    assert!(ledger.records().unwrap().is_empty());
  }
  #[test]
  fn optional_missing_read_is_rechecked() {
    let root = directory();
    let path = root.join("optional.txt");
    let ledger = SourceObservationLedger::default();
    assert_eq!(
      ledger.read_file("optional", &path).unwrap_err().kind(),
      io::ErrorKind::NotFound
    );
    ledger.verify().unwrap();
    fs::write(&path, b"new").unwrap();
    assert!(ledger.verify().is_err());
    fs::remove_dir_all(root).unwrap();
  }
  #[test]
  fn conflicting_readers_are_sticky_even_after_restoring_original_bytes() {
    let root = directory();
    let path = root.join("source.txt");
    let ledger = SourceObservationLedger::default();
    fs::write(&path, b"a").unwrap();
    ledger.read_file("one", &path).unwrap();
    fs::write(&path, b"b").unwrap();
    assert!(ledger.read_file("two", &path).is_err());
    fs::write(&path, b"a").unwrap();
    assert!(ledger.verify().is_err());
    fs::remove_dir_all(root).unwrap();
  }
  #[test]
  fn sealed_generation_refuses_new_reads_and_stale_derived_links() {
    let root = directory();
    let path = root.join("source.txt");
    fs::write(&path, b"a").unwrap();
    let ledger = SourceObservationLedger::default();
    ledger.read_file("one", &path).unwrap();
    let old = ledger.generation();
    assert!(ledger.sealed_records().is_err());
    ledger.seal().unwrap();
    assert_eq!(ledger.sealed_records().unwrap().len(), 1);
    assert!(ledger.read_file("two", &path).is_err());
    assert!(ledger.sealed_records().is_err());
    ledger.begin_generation();
    assert!(ledger.consume_derived(old, "reader", "producer").is_err());
    ledger.verify().unwrap();
    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn invalid_cache_proof_does_not_pollute_active_generation() {
    let root = directory();
    let path = root.join("cached.txt");
    fs::write(&path, b"old").unwrap();
    let previous = SourceObservationLedger::default();
    previous.read_file("cached", &path).unwrap();
    let records = previous.records().unwrap();
    fs::write(&path, b"new").unwrap();
    let current = SourceObservationLedger::default();
    assert!(current.restore_verified(&records).is_err());
    assert!(current.records().unwrap().is_empty());
    current.read_file("rebuilt", &path).unwrap();
    current.seal().unwrap();
    fs::remove_dir_all(root).unwrap();
  }

  #[test]
  fn extended_windows_paths_use_the_same_evidence_identity() {
    assert_eq!(
      text(Path::new(r"\\?\C:\work\source.ts")).unwrap(),
      "C:/work/source.ts"
    );
    assert_eq!(
      text(Path::new(r"\\?\UNC\host\share\source.ts")).unwrap(),
      "//host/share/source.ts"
    );
  }
}
