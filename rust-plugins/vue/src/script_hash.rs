use farmfe_core::error::CompilationError;

pub fn script_hash(content: &str) -> Result<String, CompilationError> {
  let mut errors = vec![];
  let descriptor = fervid_parser::SfcParser::new(content, &mut errors)
    .parse_sfc()
    .map_err(|error| {
      CompilationError::GenericError(format!("Read Vue script identity: {error:?}"))
    })?;
  let mut blocks = vec![];
  for script in descriptor
    .script_legacy
    .iter()
    .chain(descriptor.script_setup.iter())
  {
    let start = script.span.lo.0.checked_sub(1);
    let end = script.span.hi.0.checked_sub(1);
    let block = start
      .zip(end)
      .and_then(|(start, end)| content.get(start as usize..end as usize))
      .ok_or_else(|| {
        CompilationError::GenericError("Invalid Vue script source range".to_string())
      })?;
    blocks.push(block);
  }
  Ok(crate::descriptor::content_hash(&blocks.join("\0")).to_string())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn template_and_scoped_style_edits_preserve_script_identity() {
    let script = "<script setup lang='ts'>const stock = 17;</script>";
    let first =
      format!("<template><p>one</p></template>{script}<style scoped>p{{color:red}}</style>");
    let second = format!(
      "<template><p>库存 workspace</p></template>{script}<style scoped>p{{color:blue}}</style>"
    );
    assert_eq!(script_hash(&first).unwrap(), script_hash(&second).unwrap());
    assert_ne!(
      script_hash(&first).unwrap(),
      script_hash(&first.replace("stock = 17", "stock = 42")).unwrap()
    );
  }
}
