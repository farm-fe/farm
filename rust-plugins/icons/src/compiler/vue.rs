use super::CompilerParams;
use fervid::{compile, CompileOptions, CompileResult};

pub fn vue_compiler(param: CompilerParams) -> String {
  let CompilerParams { svg, svg_name, .. } = param;
  let code = format!("<template>{svg}</template>");
  let CompileResult { code, .. } = compile(
    &code,
    CompileOptions {
      filename: std::borrow::Cow::Borrowed(&svg_name.unwrap_or("Index".to_string())),
      id: std::borrow::Cow::Borrowed("index"),
      is_prod: Some(true),
      ssr: Some(false),
      source_map: None,
      gen_default_as: None,
    },
  )
  .unwrap();
  code
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::env;

  #[test]
  fn test_vue_compiler() {
    let current_dir = env::current_dir().unwrap();
    let root_path = current_dir.join("playground").to_string_lossy().to_string();
    let svg = r#"<template>
  <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="2em">
    <path fill="currentColor"
      d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm-2-6h4v-2h-4v2z" />
  </svg>
</template>"#;
    let params = CompilerParams {
      svg: svg.to_string(),
      root_path: Some(root_path),
      svg_name: Some("abc".to_string()),
    };
    let result = vue_compiler(params);
    println!("{result}")
  }
}
