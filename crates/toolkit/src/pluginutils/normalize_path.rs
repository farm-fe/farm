pub fn normalize_path(path: &str) -> String {
  let windows_separator = "\\";
  path.split(windows_separator).collect::<Vec<_>>().join("/")
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_normalize_path() {
    assert_eq!(normalize_path("folder\\file.txt"), "folder/file.txt");
    assert_eq!(normalize_path("folder\\\\file.txt"), "folder//file.txt");
    assert_eq!(normalize_path("C:\\folder\\file.txt"), "C:/folder/file.txt");
    assert_eq!(normalize_path("C:/folder/file.txt"), "C:/folder/file.txt");
    assert_eq!(
      normalize_path("/folder\\subfolder\\file.txt"),
      "/folder/subfolder/file.txt"
    );
  }
}
