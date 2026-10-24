//! Scope and value checks for controlled positive and negative input receipts.
use crate::{error, source::public_path, valid_hash};
use farmfe_core::context::source_observation::{
  InputObservation, ObservedValue, SourceObservationLedger,
};
use std::{fs, path::Path};

/// Validate path identities before allowing a persisted observation into a host.
pub fn validate_observation_scope(records: &[InputObservation]) -> Result<(), String> {
  for record in records {
    if record.reader.is_empty() {
      return Err(error("input observation has no reader"));
    }
    if let ObservedValue::Environment { hash, .. } = &record.value {
      if record.query.is_empty() || record.query.contains(['=', '\0']) || !valid_hash(hash) {
        return Err(error("invalid environment observation"));
      }
      continue;
    }
    let query = Path::new(&record.query);
    public_path(query)?;
    // Negative paths still have an existing ancestor. Check its real identity so
    // a symlink into private tooling cannot hide behind a nonexistent suffix.
    let mut ancestor = query;
    loop {
      match fs::canonicalize(ancestor) {
        Ok(canonical) => {
          public_path(&canonical)?;
          break;
        }
        Err(failure) if failure.kind() == std::io::ErrorKind::NotFound => {
          ancestor = ancestor
            .parent()
            .ok_or_else(|| error("observation has no existing ancestor"))?;
        }
        Err(failure) => return Err(error(failure)),
      }
    }
    match &record.value {
      ObservedValue::File { canonical, hash } => {
        public_path(Path::new(canonical))?;
        if !valid_hash(hash) {
          return Err(error("invalid observed file hash"));
        }
      }
      ObservedValue::Metadata {
        canonical: Some(canonical),
        ..
      }
      | ObservedValue::Directory { canonical, .. } => public_path(Path::new(canonical))?,
      _ => {}
    }
  }
  Ok(())
}

pub fn verify_observations(records: &[InputObservation]) -> Result<(), String> {
  validate_observation_scope(records)?;
  SourceObservationLedger::verify_records(records).map_err(error)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn complete_observations_recheck_file_negative_directory_and_environment() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("source.css");
    fs::write(&file, "color:purple").unwrap();
    let ledger = SourceObservationLedger::default();
    ledger.read_file("css", &file).unwrap();
    ledger
      .metadata("css", &root.path().join("optional.json"))
      .unwrap();
    ledger.read_directory("css", root.path()).unwrap();
    ledger
      .read_environment("css", "FARM_DOCS_SNAPSHOT_OBSERVATION_TEST")
      .unwrap();
    let records = ledger.records().unwrap();
    assert_eq!(records.len(), 4);
    verify_observations(&records).unwrap();
    fs::write(root.path().join("optional.json"), "{}").unwrap();
    assert!(verify_observations(&records).is_err());
  }

  #[test]
  fn even_missing_private_queries_are_rejected() {
    let root = tempfile::tempdir().unwrap();
    let ledger = SourceObservationLedger::default();
    ledger
      .metadata("css", &root.path().join(".farm-docs/absent.json"))
      .unwrap();
    assert!(validate_observation_scope(&ledger.records().unwrap()).is_err());
  }
}
