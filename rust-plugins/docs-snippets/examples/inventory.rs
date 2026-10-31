//! Read-only evidence tool. This does not migrate sources or compile site output.
use farmfe_core::{
  module::ModuleId,
  swc_common::{GLOBALS, Globals},
  swc_ecma_ast::{EsVersion, ModuleDecl, ModuleItem},
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_plugin_docs_snippets::inventory::inspect_with_resolver;
use farmfe_toolkit::script::{parse_module, swc_try_with::resolve_module_mark};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::{
  fs,
  io::Write,
  path::{Path, PathBuf},
  sync::Arc,
};

fn collect(root: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
  for entry in fs::read_dir(root)? {
    let entry = entry?;
    let kind = entry.file_type()?;
    if kind.is_dir() {
      collect(&entry.path(), files)?;
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

fn resolve_theme_import(root: &Path, importer: &Path, source: &str) -> Option<String> {
  if !source.starts_with("./") && !source.starts_with("../") {
    return None;
  }
  let requested = importer.parent()?.join(source);
  let mut candidates = vec![requested.clone()];
  for extension in ["tsx", "ts", "jsx", "js"] {
    candidates.push(requested.with_extension(extension));
    candidates.push(requested.join(format!("index.{extension}")));
  }
  let theme_root = fs::canonicalize(root.join("src/theme")).ok()?;
  for candidate in candidates {
    let Ok(candidate) = fs::canonicalize(candidate) else {
      continue;
    };
    if !candidate.is_file() {
      continue;
    }
    // Respect the first existing module, including one outside the theme root.
    let relative = candidate.strip_prefix(&theme_root).ok()?;
    let mut module = relative.to_path_buf();
    module.set_extension("");
    if module.file_name().is_some_and(|name| name == "index") {
      module.pop();
    }
    return Some(format!(
      "@site/src/theme/{}",
      module.to_string_lossy().replace('\\', "/")
    ));
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn resolves_only_existing_theme_modules_under_the_website_root() -> std::io::Result<()> {
    let nonce = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .map_err(std::io::Error::other)?
      .as_nanos();
    let root = std::env::temp_dir().join(format!(
      "farm-docs-inventory-{}-{nonce}",
      std::process::id()
    ));
    fs::create_dir(&root)?;
    fs::create_dir_all(root.join("src/theme/PackageManagerTabs"))?;
    fs::create_dir_all(root.join("docs/nested"))?;
    fs::write(
      root.join("src/theme/PackageManagerTabs/index.tsx"),
      "export const PackageManagerTabs = 1;",
    )?;
    fs::write(
      root.join("docs/nested/PackageManagerTabs.tsx"),
      "export default 1;",
    )?;
    let importer = root.join("docs/nested/page.mdx");
    let root = fs::canonicalize(root)?;
    assert_eq!(
      resolve_theme_import(&root, &importer, "../../src/theme/PackageManagerTabs"),
      Some("@site/src/theme/PackageManagerTabs".into())
    );
    assert_eq!(
      resolve_theme_import(
        &root,
        &importer,
        "../../src/theme/PackageManagerTabs/index.tsx"
      ),
      Some("@site/src/theme/PackageManagerTabs".into())
    );
    assert_eq!(
      resolve_theme_import(&root, &importer, "./PackageManagerTabs"),
      None
    );
    assert_eq!(
      resolve_theme_import(&root, &importer, "../../src/theme/Missing"),
      None
    );
    assert_eq!(
      resolve_theme_import(&root, &importer, "../../../outside/PackageManagerTabs"),
      None
    );
    assert_eq!(
      resolve_theme_import(&root, &importer, "some-package/PackageManagerTabs"),
      None
    );
    fs::remove_dir_all(root)?;
    Ok(())
  }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  let args = std::env::args_os().skip(1).collect::<Vec<_>>();
  if args.len() != 2 {
    return Err("usage: inventory <website-directory> <new-report.json>".into());
  }
  let root = fs::canonicalize(&args[0])?;
  let mut output = fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&args[1])?;
  let mut files = vec![];
  for directory in ["docs", "versioned_docs", "i18n", "blog", "src/pages"] {
    collect(&root.join(directory), &mut files)?;
  }
  files.sort();
  let mut rows = vec![];
  let mut failures = 0;
  for file in files {
    let source = fs::read_to_string(&file)?;
    let relative = file
      .strip_prefix(&root)?
      .to_string_lossy()
      .replace('\\', "/");
    let compiled = farmfe_docs_content::compile_content_with_format(
      &source,
      &mdxjs::Options {
        jsx: true,
        filepath: Some(file.to_string_lossy().into_owned()),
        ..Default::default()
      },
      true,
      None,
      farmfe_docs_content::ContentFormat::Auto,
    );
    let result = compiled.and_then(|compiled| {
      let id = ModuleId::from(relative.as_str());
      let mut ast = parse_module(
        &id,
        Arc::new(compiled.code),
        Syntax::Es(EsSyntax {
          jsx: true,
          ..Default::default()
        }),
        EsVersion::latest(),
      )
      .map_err(|error| error.to_string())?
      .ast;
      let globals = Globals::new();
      resolve_module_mark(&mut ast, false, &globals);
      let resolutions = ast
        .body
        .iter()
        .filter_map(|item| {
          let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item else {
            return None;
          };
          let source = import.src.value.to_string_lossy().into_owned();
          (source.starts_with("./") || source.starts_with("../")).then(|| {
            let resolved = resolve_theme_import(&root, &file, &source);
            (source, resolved)
          })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
      let inventory = GLOBALS.set(&globals, || {
        inspect_with_resolver(&ast, |source| resolutions.get(source).cloned().flatten())
      });
      Ok((inventory, resolutions))
    });
    let mut row =
      json!({"path":relative,"sha256":format!("{:x}",Sha256::digest(source.as_bytes()))});
    match result {
      Ok((inventory, resolutions)) => {
        row["success"] = json!(true);
        row["inventory"] = serde_json::to_value(inventory)?;
        row["relativeImportResolutions"] = serde_json::to_value(resolutions)?;
      }
      Err(error) => {
        failures += 1;
        row["success"] = json!(false);
        row["error"] = json!(error);
      }
    }
    rows.push(row);
  }
  let report = json!({"scope":"Rust AST of actual Docs-generated JSX, resolver-bound aliases and canonical existing relative theme module identities; original import strings retained; no author source edits, migration or site acceptance", "files":rows.len(), "failures":failures, "rows":rows});
  output.write_all(serde_json::to_string_pretty(&report)?.as_bytes())?;
  output.write_all(b"\n")?;
  println!(
    "Inspected {} content files, {failures} failures",
    report["files"]
  );
  if failures != 0 {
    return Err("corpus inventory has failures; inspect report".into());
  }
  Ok(())
}
