//! One-time, report-bound author migration. Compilation stays in native plugins.
use farmfe_core::{
  module::ModuleId,
  swc_ecma_ast::*,
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::script::parse_module;
use markdown::mdast::{AttributeContent, AttributeValue, Node};
use serde_json::{Value, json};
use std::{
  collections::{BTreeMap, BTreeSet},
  sync::Arc,
};

struct Edit {
  start: usize,
  end: usize,
  text: String,
}

fn target(import: &Value) -> Result<(&'static str, &'static str), String> {
  match (
    import["resolvedSource"].as_str(),
    import["imported"].as_str(),
  ) {
    (Some("@theme/Tabs"), Some("default")) => Ok(("Tabs", "@farmfe/docs/mdx")),
    (Some("@theme/TabItem"), Some("default")) => Ok(("TabItem", "@farmfe/docs/mdx")),
    (Some("@theme/CodeBlock"), Some("default")) => Ok(("CodeBlock", "@farmfe/docs/snippets")),
    (Some("@site/src/theme/PackageManagerTabs"), Some("PackageManagerTabs")) => {
      Ok(("CommandTabs", "@farmfe/docs/snippets"))
    }
    _ => Err(format!("Unrecognized migration import: {import}")),
  }
}

fn visit(
  node: &Node,
  source: &str,
  imports: &[Value],
  uses: &[Value],
  commands: &Value,
  seen_imports: &mut BTreeSet<String>,
  seen_uses: &mut usize,
  edits: &mut Vec<Edit>,
) -> Result<(), String> {
  if let Node::MdxjsEsm(esm) = node {
    let position = esm.position.as_ref().ok_or("ESM missing original range")?;
    if source.get(position.start.offset..position.end.offset) != Some(esm.value.as_str()) {
      return Err("Non-contiguous ESM requires explicit source mapping".into());
    }
    let parsed = parse_module(
      &ModuleId::from("author-esm.js"),
      Arc::new(esm.value.clone()),
      Syntax::Es(EsSyntax::default()),
      EsVersion::latest(),
    )
    .map_err(|e| e.to_string())?;
    for item in &parsed.ast.body {
      let ModuleItem::ModuleDecl(ModuleDecl::Import(declaration)) = item else {
        continue;
      };
      let raw_source = declaration.src.value.to_string_lossy();
      let selected = imports
        .iter()
        .filter(|i| i["source"].as_str() == Some(raw_source.as_ref()))
        .collect::<Vec<_>>();
      if selected.is_empty() {
        continue;
      }
      if declaration.type_only || selected.len() != declaration.specifiers.len() {
        return Err("Mixed or type-only migration import".into());
      }
      let mut replacements = vec![];
      for specifier in &declaration.specifiers {
        let local = specifier.local().sym.to_string();
        let record = selected
          .iter()
          .find(|i| i["local"].as_str() == Some(&local))
          .ok_or("Import binding mismatch")?;
        if record["typeOnly"].as_bool() != Some(false) || !seen_imports.insert(local.clone()) {
          return Err("Duplicate or type-only binding".into());
        }
        let (name, module) = target(record)?;
        replacements.push(format!("import {{ {name} as {local} }} from {module:?};"));
      }
      let base = parsed
        .source_map
        .lookup_source_file(declaration.span.lo)
        .start_pos
        .0;
      let start = position.start.offset + (declaration.span.lo.0 - base) as usize;
      let end = position.start.offset + (declaration.span.hi.0 - base) as usize;
      edits.push(Edit {
        start,
        end,
        text: replacements.join("\n"),
      });
    }
  }
  let element = match node {
    Node::MdxJsxFlowElement(e) => Some((&e.name, &e.attributes, &e.position)),
    Node::MdxJsxTextElement(e) => Some((&e.name, &e.attributes, &e.position)),
    _ => None,
  };
  if let Some((Some(name), attributes, position)) = element {
    if let Some(import) = imports.iter().find(|i| i["local"].as_str() == Some(name)) {
      let (component, _) = target(import)?;
      let usage = uses
        .get(*seen_uses)
        .ok_or("More author uses than bound AST uses")?;
      let expected_component = if component == "CommandTabs" {
        "PackageManagerTabs"
      } else {
        component
      };
      if usage["local"].as_str() != Some(name)
        || usage["component"].as_str() != Some(expected_component)
        || usage["dynamic"].as_bool() != Some(false)
      {
        return Err("Author/binding order mismatch or dynamic use".into());
      }
      *seen_uses += 1;
      let position = position.as_ref().ok_or("Component missing source range")?;
      if component == "CommandTabs" {
        if attributes.len() != 1 || node.children().is_some_and(|c| !c.is_empty()) {
          return Err("Unsupported package command shape".into());
        }
        let AttributeContent::Property(attribute) = &attributes[0] else {
          return Err("Dynamic command attribute".into());
        };
        let Some(AttributeValue::Literal(command)) = &attribute.value else {
          return Err("Command must be an authored literal".into());
        };
        if attribute.name != "command" || usage["attributes"]["command"].as_str() != Some(command) {
          return Err("Command inventory mismatch".into());
        }
        let mapping = commands
          .get(command)
          .and_then(Value::as_object)
          .ok_or_else(|| format!("Command outside explicit migration table: {command}"))?;
        if mapping.len() != 4
          || ["npm", "pnpm", "yarn", "bun"].iter().any(|k| {
            mapping
              .get(*k)
              .and_then(Value::as_str)
              .is_none_or(str::is_empty)
          })
        {
          return Err("Invalid explicit four-tool command mapping".into());
        }
        edits.push(Edit {
          start: position.start.offset,
          end: position.end.offset,
          text: format!(
            "<{name} commands={{{}}} />",
            serde_json::to_string(mapping).map_err(|e| e.to_string())?
          ),
        });
      } else if component == "CodeBlock" {
        if !attributes.is_empty() || usage["attributes"] != json!({}) {
          return Err("CodeBlock options need explicit migration support".into());
        }
        let code = usage["code"].as_str().ok_or("Missing static code")?;
        edits.push(Edit {
          start: position.start.offset,
          end: position.end.offset,
          text: format!(
            "<{name} language=\"javascript\" code={{{}}} />",
            serde_json::to_string(code).map_err(|e| e.to_string())?
          ),
        });
        return Ok(());
      }
    }
  }
  if let Some(children) = node.children() {
    for child in children {
      visit(
        child,
        source,
        imports,
        uses,
        commands,
        seen_imports,
        seen_uses,
        edits,
      )?;
    }
  }
  Ok(())
}

fn migrate(source: &str, inventory: &Value, commands: &Value) -> Result<String, String> {
  let imports = inventory["imports"].as_array().ok_or("Missing imports")?;
  let uses = inventory["uses"].as_array().ok_or("Missing uses")?;
  let mut locals = BTreeMap::new();
  for import in imports {
    target(import)?;
    let local = import["local"].as_str().ok_or("Missing import local")?;
    if locals.insert(local, import).is_some() {
      return Err("Duplicate local import".into());
    }
  }
  if uses.iter().any(|u| u["dynamic"].as_bool() != Some(false)) {
    return Err("Dynamic uses cannot be migrated".into());
  }
  let mut options = mdxjs::Options::default();
  options.parse.constructs.frontmatter = true;
  let tree = mdxjs::mdast_util_from_mdx(source, &options).map_err(|e| e.to_string())?;
  let mut seen_imports = BTreeSet::new();
  let mut seen_uses = 0;
  let mut edits = vec![];
  visit(
    &tree,
    source,
    imports,
    uses,
    commands,
    &mut seen_imports,
    &mut seen_uses,
    &mut edits,
  )?;
  if seen_imports.len() != imports.len() || seen_uses != uses.len() {
    return Err("Inventory coverage mismatch; no changes applied".into());
  }
  edits.sort_by_key(|e| e.start);
  let mut end = 0;
  for edit in &edits {
    if edit.start < end || source.get(edit.start..edit.end).is_none() {
      return Err("Overlapping or invalid UTF-8 source ranges".into());
    }
    end = edit.end;
  }
  let mut output = source.to_owned();
  for edit in edits.into_iter().rev() {
    output.replace_range(edit.start..edit.end, &edit.text);
  }
  Ok(output)
}

#[derive(serde::Serialize, serde::Deserialize)]
struct FileChange {
  path: String,
  before: String,
  after: String,
  before_sha256: String,
  after_sha256: String,
}

fn digest(source: &str) -> String {
  use sha2::{Digest, Sha256};
  format!("{:x}", Sha256::digest(source.as_bytes()))
}

fn validate_migrated(source: &str, original: &Value, commands: &Value) -> Result<(), String> {
  use farmfe_core::swc_common::{GLOBALS, Globals};
  use farmfe_plugin_docs_snippets::{
    inventory::inspect,
    transform::{Options, lower},
  };
  use farmfe_toolkit::script::swc_try_with::resolve_module_mark;
  let compiled = farmfe_docs_content::compile_content_with_format(
    source,
    &mdxjs::Options {
      jsx: true,
      ..Default::default()
    },
    false,
    None,
    farmfe_docs_content::ContentFormat::Auto,
  )?;
  let mut ast = parse_module(
    &ModuleId::from("migrated-author.jsx"),
    Arc::new(compiled.code),
    Syntax::Es(EsSyntax {
      jsx: true,
      ..Default::default()
    }),
    EsVersion::latest(),
  )
  .map_err(|e| e.to_string())?
  .ast;
  let globals = Globals::new();
  resolve_module_mark(&mut ast, false, &globals);
  GLOBALS.set(&globals,|| {
    let actual=inspect(&ast);
    if actual.imports.iter().any(|i|i["source"].as_str().is_none_or(|s|!s.starts_with("@farmfe/docs/"))){return Err("Old component import survived migration".into());}
    let expected=original["uses"].as_array().ok_or("Missing original uses")?;
    if actual.uses.len()!=expected.len(){return Err("Migrated component use count changed".into());}
    for (before,after) in expected.iter().zip(&actual.uses) {
      let name=before["component"].as_str().ok_or("Missing component")?;
      let renamed=if name=="PackageManagerTabs"{"CommandTabs"}else{name};
      if after["component"]!=renamed || after["local"]!=before["local"] || after["dynamic"]!=false {return Err("Migrated component identity changed".into());}
      let attributes=match name {
        "PackageManagerTabs"=>json!({"commands":commands.get(before["attributes"]["command"].as_str().ok_or("Missing command")?).ok_or("Unknown command")?}),
        "CodeBlock"=>json!({"language":"javascript","code":before["code"]}),
        _=>before["attributes"].clone(),
      };
      if after["attributes"]!=attributes{return Err("Migrated static values changed".into());}
    }
    let transformed=lower(&mut ast,&Options::default())?.transformed;
    let expected_snippets=expected.iter().filter(|u|u["component"]=="PackageManagerTabs" || u["component"]=="CodeBlock").count();
    if transformed!=expected_snippets{return Err("Rust snippet transform coverage changed".into());}
    Ok(())
  })
}

fn author_path(root: &std::path::Path, relative: &str) -> Result<std::path::PathBuf, String> {
  use std::path::Component;
  let path = std::path::Path::new(relative);
  if path
    .components()
    .any(|c| !matches!(c, Component::Normal(_)))
    || !(relative.starts_with("docs/")
      || relative.starts_with("versioned_docs/")
      || relative.starts_with("i18n/")
      || relative.starts_with("blog/")
      || relative.starts_with("src/pages/"))
  {
    return Err(format!("Invalid author path {relative}"));
  }
  let root = std::fs::canonicalize(root).map_err(|e| e.to_string())?;
  let resolved = std::fs::canonicalize(root.join(path)).map_err(|e| e.to_string())?;
  if !resolved.starts_with(root) || !resolved.is_file() {
    return Err(format!("Author path escaped root: {relative}"));
  }
  Ok(resolved)
}

fn plan_files(
  root: &std::path::Path,
  report: &Value,
  commands: &Value,
) -> Result<Vec<FileChange>, String> {
  let rows = report["rows"].as_array().ok_or("Missing report rows")?;
  if report["failures"] != 0 || report["files"].as_u64() != Some(rows.len() as u64) {
    return Err("Incomplete inventory report".into());
  }
  let mut seen = BTreeSet::new();
  let mut files = vec![];
  for row in rows {
    let path = row["path"].as_str().ok_or("Missing author path")?;
    if !seen.insert(path) || row["success"] != true {
      return Err("Duplicate or failed inventory row".into());
    }
    let before = std::fs::read_to_string(author_path(root, path)?).map_err(|e| e.to_string())?;
    if row["sha256"].as_str() != Some(&digest(&before)) {
      return Err(format!("Author changed since inventory: {path}"));
    }
    let imports = row["inventory"]["imports"]
      .as_array()
      .ok_or("Missing import inventory")?;
    let after = if imports.is_empty() {
      before.clone()
    } else {
      migrate(&before, &row["inventory"], commands).map_err(|e| format!("{path}: {e}"))?
    };
    if !imports.is_empty() {
      validate_migrated(&after, &row["inventory"], commands).map_err(|e| format!("{path}: {e}"))?;
    }
    files.push(FileChange {
      path: path.to_owned(),
      before_sha256: digest(&before),
      after_sha256: digest(&after),
      before,
      after,
    });
  }
  Ok(files)
}

fn apply_files(root: &std::path::Path, files: &[FileChange]) -> Result<(), String> {
  let mut paths = vec![];
  let mut seen = BTreeSet::new();
  for file in files {
    if !seen.insert(&file.path)
      || digest(&file.before) != file.before_sha256
      || digest(&file.after) != file.after_sha256
    {
      return Err("Invalid migration plan hashes or duplicate paths".into());
    }
    let path = author_path(root, &file.path)?;
    let current = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    if current != file.before {
      return Err(format!("Author changed since plan: {}", file.path));
    }
    paths.push(path);
  }
  let mut written = vec![];
  for (index, file) in files.iter().enumerate() {
    if file.before == file.after {
      continue;
    }
    written.push(index);
    let result = std::fs::write(&paths[index], &file.after).and_then(|()| {
      let content = std::fs::read_to_string(&paths[index])?;
      if content != file.after {
        return Err(std::io::Error::other("Post-write verification failed"));
      }
      Ok(())
    });
    if let Err(error) = result {
      let mut rollback_errors = vec![];
      for &previous in written.iter().rev() {
        if let Err(e) = std::fs::write(&paths[previous], &files[previous].before) {
          rollback_errors.push(format!("{}: {e}", files[previous].path));
        }
      }
      return Err(format!(
        "Write failed: {error}; rollback errors: {rollback_errors:?}"
      ));
    }
  }
  Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
  use std::{fs, io::Write};
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  match args.first().map(String::as_str) {
    Some("plan") if args.len()==5 => {
      let root=fs::canonicalize(&args[1])?;
      let report_source=fs::read_to_string(&args[2])?;
      let table_source=fs::read_to_string(&args[3])?;
      let files=plan_files(&root,&serde_json::from_str(&report_source)?,&serde_json::from_str(&table_source)?)?;
      let changed=files.iter().filter(|f|f.before!=f.after).count();
      let plan=json!({"status":"AUTHOR_MIGRATION_PLANNED","root":root,"reportSha256":digest(&report_source),"commandTableSha256":digest(&table_source),"checkedFiles":files.len(),"changedFiles":changed,"files":files});
      let mut output=fs::OpenOptions::new().write(true).create_new(true).open(&args[4])?;
      output.write_all(serde_json::to_string_pretty(&plan)?.as_bytes())?;
      println!("Planned {changed} changed files; no author writes");
    }
    Some("apply") if args.len()==4 => {
      let root=fs::canonicalize(&args[1])?;
      let plan_source=fs::read_to_string(&args[2])?;
      let plan:Value=serde_json::from_str(&plan_source)?;
      if plan["status"]!="AUTHOR_MIGRATION_PLANNED" || plan["root"]!=json!(root){return Err("Plan root or status mismatch".into());}
      let files:Vec<FileChange>=serde_json::from_value(plan["files"].clone())?;
      let mut output=fs::OpenOptions::new().write(true).create_new(true).open(&args[3])?;
      let result=apply_files(&root,&files);
      let receipt=json!({"status":if result.is_ok(){"AUTHOR_MIGRATION_APPLIED"}else{"AUTHOR_MIGRATION_FAILED"},"planSha256":digest(&plan_source),"checkedFiles":files.len(),"changedFiles":files.iter().filter(|f|f.before!=f.after).count(),"error":result.as_ref().err(),"siteAcceptance":false});
      output.write_all(serde_json::to_string_pretty(&receipt)?.as_bytes())?;
      result?;
      println!("Author migration applied; website/native acceptance still required");
    }
    _=>return Err("usage: migrate plan <website> <inventory.json> <commands.json> <new-plan.json> | migrate apply <website> <plan.json> <new-receipt.json>".into()),
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  fn inventory() -> Value {
    json!({"imports":[
      {"source":"../../src/theme/PackageManagerTabs", "resolvedSource":"@site/src/theme/PackageManagerTabs", "local":"PM", "imported":"PackageManagerTabs", "typeOnly":false},
      {"source":"@theme/CodeBlock", "resolvedSource":"@theme/CodeBlock", "local":"CB", "imported":"default", "typeOnly":false}
    ], "uses":[
      {"component":"PackageManagerTabs", "local":"PM", "attributes":{"command":"pnpm dev"}, "dynamic":false},
      {"component":"CodeBlock", "local":"CB", "attributes":{}, "code":"const 中文 = '<&>';\n", "dynamic":false}
    ]})
  }

  fn commands() -> Value {
    json!({"pnpm dev":{"npm":"npm dev", "pnpm":"pnpm dev", "yarn":"yarn dev", "bun":"bun dev"}})
  }

  #[test]
  fn explicit_website_table_preserves_all_legacy_skip_true_tool_outputs() {
    let table: Value = serde_json::from_str(include_str!(
      "../../../docs/farm-docs/website-command-migration.json"
    ))
    .unwrap();
    let entries = table.as_object().unwrap();
    assert_eq!(entries.len(), 26);
    for (command, mapping) in entries {
      for tool in ["npm", "pnpm", "yarn", "bun"] {
        // QA oracle for the old component's ASCII JS \b tool replacement.
        // This is never used by the migration or any compilation plugin.
        let mut expected = String::new();
        let mut word = String::new();
        for ch in command.chars().chain(std::iter::once('\0')) {
          if ch.is_ascii_alphanumeric() || ch == '_' {
            word.push(ch);
            continue;
          }
          if ["npm", "pnpm", "yarn", "bun"].contains(&word.as_str()) {
            expected.push_str(tool);
          } else {
            expected.push_str(&word);
          }
          word.clear();
          if ch != '\0' {
            expected.push(ch);
          }
        }
        assert_eq!(
          mapping[tool].as_str(),
          Some(expected.as_str()),
          "{command} / {tool}"
        );
      }
    }
  }

  #[test]
  fn migrates_aliases_and_preserves_unedited_crlf_unicode_and_fenced_examples() {
    let prefix = "---\r\ntitle: 中文\r\n---\r\n\r\n";
    let suffix = "\r\n\r\n[链接](/zh/docs/)\r\n\r\n```jsx\r\nimport PM from '@theme/PackageManagerTabs';\r\n<PM command='pnpm dev' />\r\n```\r\n";
    let source = format!(
      "{prefix}import {{ PackageManagerTabs as PM }} from '../../src/theme/PackageManagerTabs';\r\nimport CB from '@theme/CodeBlock';\r\n\r\n<PM command='pnpm dev' />\r\n\r\n<CB>{{`const 中文 = '<&>';\n`}}</CB>{suffix}"
    );
    let output = migrate(&source, &inventory(), &commands()).unwrap();
    assert!(output.starts_with(prefix));
    assert!(output.ends_with(suffix));
    assert!(output.contains("import { CommandTabs as PM } from \"@farmfe/docs/snippets\";"));
    assert!(output.contains("import { CodeBlock as CB } from \"@farmfe/docs/snippets\";"));
    assert!(output.contains("<PM commands={{"));
    assert!(output.contains("<CB language=\"javascript\" code={\"const 中文 = '<&>';\\n\"} />"));
    assert!(!output.contains("<PM command='pnpm dev' />\r\n\r\n<CB>"));
  }

  #[test]
  fn rejects_unknown_commands_and_inventory_mismatches_without_a_partial_result() {
    let source = "import {PackageManagerTabs as PM} from '../../src/theme/PackageManagerTabs';\n\n<PM command='pnpm dev' />\n";
    assert!(migrate(source, &inventory(), &json!({})).is_err());
    assert!(migrate(source, &inventory(), &commands()).is_err());
    let mut dynamic = inventory();
    dynamic["uses"][0]["dynamic"] = json!(true);
    assert!(migrate(source, &dynamic, &commands()).is_err());
  }

  #[test]
  fn preflights_every_author_hash_before_any_write_and_applies_a_valid_plan() {
    let root = std::env::temp_dir().join(format!(
      "farm-docs-author-{}-{}",
      std::process::id(),
      std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
    ));
    std::fs::create_dir_all(root.join("docs")).unwrap();
    let a = "import CodeBlock from '@theme/CodeBlock';\n\n<CodeBlock>{'one'}</CodeBlock>\n";
    let b = "Unchanged 中文\r\n";
    std::fs::write(root.join("docs/a.mdx"), a).unwrap();
    std::fs::write(root.join("docs/b.md"), b).unwrap();
    let report = json!({"files":2,"failures":0,"rows":[
      {"path":"docs/a.mdx","success":true,"sha256":digest(a),"inventory":{"imports":[{"source":"@theme/CodeBlock","resolvedSource":"@theme/CodeBlock","imported":"default","local":"CodeBlock","typeOnly":false}],"uses":[{"component":"CodeBlock","local":"CodeBlock","attributes":{},"code":"one","dynamic":false}]}},
      {"path":"docs/b.md","success":true,"sha256":digest(b),"inventory":{"imports":[],"uses":[]}}
    ]});
    let files = plan_files(&root, &report, &json!({})).unwrap();
    assert_eq!(files.len(), 2);
    std::fs::write(root.join("docs/b.md"), "later edit").unwrap();
    assert!(apply_files(&root, &files).is_err());
    assert_eq!(std::fs::read_to_string(root.join("docs/a.mdx")).unwrap(), a);
    assert_eq!(
      std::fs::read_to_string(root.join("docs/b.md")).unwrap(),
      "later edit"
    );
    assert!(plan_files(&root, &report, &json!({})).is_err());
    std::fs::write(root.join("docs/b.md"), b).unwrap();
    apply_files(&root, &files).unwrap();
    assert!(
      std::fs::read_to_string(root.join("docs/a.mdx"))
        .unwrap()
        .contains("@farmfe/docs/snippets")
    );
    assert_eq!(std::fs::read_to_string(root.join("docs/b.md")).unwrap(), b);
    std::fs::remove_dir_all(root).unwrap();
  }
}
