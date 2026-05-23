use super::scan_exports::Import;
pub fn stringify_imports(imports: Vec<Import>) -> String {
  let mut imports_str = String::new();
  for import in imports {
    imports_str += &import.stringify_import();
  }
  imports_str
}
