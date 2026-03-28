use crate::parser::parse::{parse_esm_imports, ESMImport};
use farmfe_core::regex::Regex;
use regress::Regex as JsRegex;

use super::scan_exports::Import;
use super::stringify_imports::stringify_imports;

const JS_VAR_DEF_REGEX: &str = r"(?:^|\s+)(?:let|const|var)\s+(?<var_name>[\w$]+)";

const JS_CLASS_DEF_REGEX: &str = r"\bclass\s+(?<class_name>[\w$]+)";

const JS_FUNC_DEF_REGEX: &str = r"(?:^|\s+)function\s+(?<func_name>[\w$]+)\s*\(";

const MATCH_RE: &str = r"(^|\.\.\.|(?:\bcase|\?)\s+|[^\w$/)]|\bextends\s+)([\w$]+)\s*(?=[.()[\]}:;?+\-*&|`<>,\n]|\b(?:instanceof|in)\b|$|(?<=extends\s+\w+)\s+\{)";

fn get_exclude_imports(content: &str, imports: Vec<Import>) -> Vec<Import> {
  let mut exclude_vars = vec![];
  let mut include_vars = vec![];
  for capture in Regex::new(JS_VAR_DEF_REGEX)
    .unwrap()
    .captures_iter(&content)
  {
    if let Some(var_name) = capture.name("var_name") {
      exclude_vars.push(var_name.as_str());
    }
  }

  for capture in Regex::new(JS_CLASS_DEF_REGEX)
    .unwrap()
    .captures_iter(&content)
  {
    if let Some(class_name) = capture.name("class_name") {
      exclude_vars.push(class_name.as_str());
    }
  }
  for capture in Regex::new(JS_FUNC_DEF_REGEX)
    .unwrap()
    .captures_iter(&content)
  {
    if let Some(func_name) = capture.name("func_name") {
      exclude_vars.push(func_name.as_str());
    }
  }
  for m in JsRegex::new(MATCH_RE).unwrap().find_iter(&content) {
    include_vars.push(content[m.range()].trim());
  }
  imports
    .into_iter()
    .filter(|item| {
      let name = item.as_name.clone().unwrap_or(item.name.clone());
      let name = &name.as_str();
      !exclude_vars.contains(name) && include_vars.contains(name)
    })
    .collect()
}

pub fn inject_imports(content: &str, imports: Vec<Import>, priority: Option<usize>, inject_at_end: bool) -> String {
  let esm_imports = parse_esm_imports(None, Some(content));
  let imports = get_exclude_imports(&content, imports)
    .into_iter()
    .filter(|import| {
      !esm_imports.iter().any(|esm_import| {
        let ESMImport {
          named_imports,
          default_import,
          namespaced_import,
          type_named_imports,
          ..
        } = esm_import;
        let Import {
          name: import_name,
          priority: import_priority,
          ..
        } = import;
        let c_priority = priority.unwrap_or(1) - import_priority;
        if let Some(named_import) = named_imports {
          let import_keys: Vec<String> = named_import.keys().cloned().collect();
          if import_keys.contains(&import_name) {
            if c_priority == 0 {
              println!(
                "{}",
                format!(
                  "Duplicated in imported, has been ignored and {} is used",
                  import_name
                )
              );
              return false;
            } else {
              return true;
            }
          }
        }
        if let Some(type_named_import) = type_named_imports {
          let import_keys: Vec<String> = type_named_import.keys().cloned().collect();
          if import_keys.contains(&import_name) {
            return true;
          }
        }
        if let Some(default_import) = default_import {
          if default_import == import_name {
            if c_priority == 0 {
              println!(
                "{}",
                format!(
                  "Duplicated in imported, has been ignored and {} is used",
                  import_name
                )
              );
              return false;
            } else {
              return true;
            }
          }
        }
        if let Some(namespaced_import) = namespaced_import {
          if namespaced_import == import_name {
            if c_priority == 0 {
              println!(
                "{}",
                format!(
                  "Duplicated in imported, has been ignored and {} is used",
                  import_name
                )
              );
              return false;
            } else {
              return true;
            }
          }
        }
        false
      })
    })
    .collect::<Vec<Import>>();
  let inject_idx = if inject_at_end {
    esm_imports.last().map_or(0, |i| i.span.hi.0 as usize)
  } else {
    0
  };
  let mut content_str = stringify_imports(imports);
  let mut content = content.to_string();
  content.insert_str(inject_idx, &content_str);
  // content_str.push_str(content);
  content
}
