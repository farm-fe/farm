use std::path::Path;

pub fn add_extension(filename: &str, ext: Option<&str>) -> String {
  let path = Path::new(filename);
  if path.extension().is_none() {
    format!("{}{}", filename, ext.unwrap_or(".js"))
  } else {
    filename.to_string()
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn add_extension_when_none_exists() {
    let filename = "index";
    let filename_with_extension = add_extension(filename, None);
    assert_eq!(filename_with_extension, "index.js");
  }

  #[test]
  fn does_not_alter_filenames_with_existing_extension() {
    let filename = "index.js";
    let filename_with_extension = add_extension(filename, Some(".ts"));
    assert_eq!(filename_with_extension, "index.js");
  }
}
