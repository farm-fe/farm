//! Compile the actual website content without resolving application imports.
//! Evidence tooling only; production content generation belongs to native plugins.
use farmfe_docs_content::{ContentFormat, HighlightOptions, compile_content_with_format};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
  env, fs,
  path::{Path, PathBuf},
  process::ExitCode,
};

fn collect(root: &Path, output: &mut Vec<PathBuf>) -> std::io::Result<()> {
  for entry in fs::read_dir(root)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    if file_type.is_dir() {
      collect(&entry.path(), output)?;
    } else if file_type.is_file()
      && entry
        .path()
        .extension()
        .is_some_and(|extension| extension == "md" || extension == "mdx")
    {
      output.push(entry.path());
    }
  }
  Ok(())
}

fn run() -> Result<bool, Box<dyn std::error::Error>> {
  let arguments: Vec<_> = env::args_os().skip(1).collect();
  if arguments.len() < 2
    || arguments[2..]
      .iter()
      .any(|arg| arg != "--highlight" && arg != "--format=mdx")
  {
    return Err(
      "Usage: audit <website-directory> <report.json> [--highlight] [--format=mdx]".into(),
    );
  }
  let highlight = arguments[2..]
    .iter()
    .any(|arg| arg == "--highlight")
    .then(HighlightOptions::default);
  let format = if arguments[2..].iter().any(|arg| arg == "--format=mdx") {
    ContentFormat::Mdx
  } else {
    ContentFormat::Auto
  };
  let root = fs::canonicalize(&arguments[0])?;
  let mut files = vec![];
  for folder in ["docs", "versioned_docs", "i18n", "blog", "src/pages"] {
    collect(&root.join(folder), &mut files)?;
  }
  files.sort();
  let mut rows = vec![];
  let mut failures = 0;
  for file in files {
    let bytes = fs::read(&file)?;
    let options = mdxjs::Options {
      filepath: Some(file.to_string_lossy().into_owned()),
      jsx: true,
      ..Default::default()
    };
    let relative = file
      .strip_prefix(&root)?
      .to_string_lossy()
      .replace('\\', "/");
    let mut row = json!({ "path": relative, "sha256": format!("{:x}", Sha256::digest(&bytes)), "bytes": bytes.len() });
    let result = String::from_utf8(bytes)
      .map_err(|error| format!("[DOCS_ENCODING] {error}"))
      .and_then(|source| {
        compile_content_with_format(&source, &options, false, highlight.as_ref(), format)
      });
    match result {
      Ok(content) => {
        row["success"] = json!(true);
        row["headings"] = json!(content.toc.len());
        row["diagnostics"] = json!(content.diagnostics);
      }
      Err(error) => {
        failures += 1;
        row["success"] = json!(false);
        row["error"] = json!(error);
      }
    }
    rows.push(row);
  }
  let report = json!({ "scope": "Rust parser/compiler corpus audit; does not prove imports, styles, source maps or browser behavior", "highlight": highlight.is_some(), "format": format, "files": rows.len(), "failures": failures, "rows": rows });
  let output = PathBuf::from(&arguments[1]);
  if let Some(parent) = output.parent() {
    fs::create_dir_all(parent)?;
  }
  fs::write(output, serde_json::to_string_pretty(&report)? + "\n")?;
  println!("Corpus: {} files, {failures} failures", report["files"]);
  Ok(failures == 0)
}

fn main() -> ExitCode {
  match run() {
    Ok(true) => ExitCode::SUCCESS,
    Ok(false) => ExitCode::FAILURE,
    Err(error) => {
      eprintln!("{error}");
      ExitCode::FAILURE
    }
  }
}
