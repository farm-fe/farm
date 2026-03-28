use crate::parser::scan_exports::Import;
use regress::Regex;
const CONTEXT_RE: &str = r#"\b_ctx\.([$\w]+)\b"#;
pub fn vue_template_addon(content: &mut String, imports: &Vec<Import>) {
  let re = Regex::new(CONTEXT_RE).unwrap();
  let mut result = String::new();
  let mut last_end = 0;

  for cap in re.find_iter(&content) {
    let start = cap.start();
    let end = cap.end();
    let import = &content[start + 5..end];
    if imports.iter().any(|item| item.name == import) {
      result.push_str(&content[last_end..start]);
      result.push_str(import);
      last_end = end;
    } else {
      result.push_str(&content[last_end..end]);
      last_end = end;
    }
  }

  result.push_str(&content[last_end..]);

  *content = result;
}

#[cfg(test)]
mod tests {
  use crate::parser::scan_dirs_exports::scan_dir_exports;
  use std::env;

  use super::*;
  #[test]
  fn test_vue_template_addon() {
    let current_dir = env::current_dir().unwrap();
    let binding = current_dir.join("playground-vue");
    let root_path = binding.to_str().unwrap();
    let imports = scan_dir_exports(root_path);
    let mut content = String::from(
      r#"
    function _sfc_render(_ctx, _cache, $props, $setup, $data, $options) {
        return _openBlock(), _createElementBlock(
          _Fragment,
          null,
          [
            _createElementVNode("div", {
              onClick: _cache[0] || (_cache[0] = (...args) => _ctx.getName && _ctx.getName(...args))
            }, [..._hoisted_3]),
            _createVNode($setup["HelloWorld"], { msg: "Farm + Vue" })
          ],
          64
          /* STABLE_FRAGMENT */
        );
      }
    "#,
    );
    vue_template_addon(&mut content, &imports);
    println!("content:{}", content);
  }
}
