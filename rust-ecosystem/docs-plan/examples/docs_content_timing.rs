//! Diagnostic only: isolate content parse/highlight/emit from route and resource planning.
use std::{
  env, fs,
  path::Path,
  time::{Duration, Instant},
};

fn collect(directory: &Path, files: &mut Vec<std::path::PathBuf>) -> std::io::Result<()> {
  for entry in fs::read_dir(directory)? {
    let entry = entry?;
    let kind = entry.file_type()?;
    if kind.is_dir() {
      if !entry
        .file_name()
        .to_string_lossy()
        .eq_ignore_ascii_case("node_modules")
        && !farmfe_docs_config::is_private_tooling_component(&entry.file_name().to_string_lossy())
      {
        collect(&entry.path(), files)?;
      }
    } else if kind.is_file()
      && entry
        .path()
        .extension()
        .is_some_and(|extension| extension == "md" || extension == "mdx")
    {
      files.push(entry.path());
    }
  }
  Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let root = fs::canonicalize(env::args().nth(1).ok_or("provide site root")?)?;
  let source = fs::read_to_string(root.join("farm-docs.site.json"))?;
  let config = farmfe_docs_config::parse_config(&source, &root)?;
  let mut rows = vec![];
  let mut parse_time = Duration::ZERO;
  let mut emit_time = Duration::ZERO;
  let total = Instant::now();
  for collection in &config.collections {
    let mut files = vec![];
    collect(&collection.directory, &mut files)?;
    files.sort();
    for file in files {
      if collection.excluded_files.contains(&file) {
        continue;
      }
      let source = fs::read_to_string(&file)?;
      let normalized = file
        .to_str()
        .ok_or("UTF-8 source path required")?
        .replace('\\', "/");
      let filepath = if let Some(unc) = normalized.strip_prefix("//?/UNC/") {
        format!("//{unc}")
      } else {
        normalized
          .strip_prefix("//?/")
          .unwrap_or(&normalized)
          .to_string()
      };
      let format = match collection.format {
        farmfe_docs_config::ContentFormat::Auto => farmfe_docs_content::ContentFormat::Auto,
        farmfe_docs_config::ContentFormat::Markdown => farmfe_docs_content::ContentFormat::Markdown,
        farmfe_docs_config::ContentFormat::Mdx => farmfe_docs_content::ContentFormat::Mdx,
      };
      let start = Instant::now();
      let parsed = farmfe_docs_content::parse_content(
        &source,
        &mdxjs::Options {
          filepath: Some(filepath.clone()),
          jsx: true,
          ..Default::default()
        },
        Some(&farmfe_docs_content::HighlightOptions::default()),
        format,
      )?;
      let parse = start.elapsed();
      parse_time += parse;
      let start = Instant::now();
      farmfe_docs_content::emit_content(parsed, true)?;
      let emit = start.elapsed();
      emit_time += emit;
      rows.push(serde_json::json!({"collection":collection.public.id,"path":filepath,"parseMs":parse.as_millis(),"emitMs":emit.as_millis()}));
    }
  }
  println!(
    "{}",
    serde_json::json!({"scope":"Rust content-stage diagnostic; no route/reference rewrite, fallback expansion, bundling or native acceptance", "documents":rows.len(),"elapsedMs":total.elapsed().as_millis(),"parseMs":parse_time.as_millis(),"emitMs":emit_time.as_millis(),"rows":rows})
  );
  Ok(())
}
