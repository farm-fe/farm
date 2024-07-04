use std::collections::HashSet;

const RESERVED_KEYWORDS: [&str; 60] = [
  "break",
  "case",
  "catch",
  "class",
  "const",
  "continue",
  "debugger",
  "default",
  "delete",
  "do",
  "else",
  "enum",
  "export",
  "extends",
  "false",
  "finally",
  "for",
  "function",
  "if",
  "import",
  "in",
  "instanceof",
  "new",
  "null",
  "return",
  "super",
  "switch",
  "this",
  "throw",
  "true",
  "try",
  "typeof",
  "var",
  "void",
  "while",
  "with",
  "as",
  "implements",
  "interface",
  "let",
  "package",
  "private",
  "protected",
  "public",
  "static",
  "yield",
  "any",
  "boolean",
  "constructor",
  "declare",
  "get",
  "module",
  "require",
  "number",
  "set",
  "string",
  "symbol",
  "type",
  "from",
  "of",
];

pub struct MinifiedIdentsGenerator {
  current_index: u32,
  top_level_idents: HashSet<String>,
}

impl MinifiedIdentsGenerator {
  pub fn new(mut top_level_idents: HashSet<String>) -> Self {
    // always skip internal reserved words
    for i in RESERVED_KEYWORDS {
      top_level_idents.insert(i.to_string());
    }
    Self {
      current_index: 0,
      top_level_idents,
    }
  }

  /**
   * generate minified ident in order:
   * a, b, c, ..., z, aa, ab, ac, ..., az, ba, bb, ..., bz, ca, cb, ..., cz, ..., za, zb, ..., zz
   */
  pub fn next(&mut self) -> String {
    let mut minified_ident = String::new();
    let mut c_index = self.current_index / 26;

    while c_index > 0 {
      let char_code = (c_index - 1) % 26 + b'a' as u32;
      minified_ident.push(char::from_u32(char_code).unwrap());

      c_index /= 26;
    }

    let c_index = self.current_index % 26 + b'a' as u32;
    minified_ident.push(char::from_u32(c_index).unwrap());
    self.current_index += 1;

    minified_ident
  }

  pub fn add_used_ident(&mut self, ident: &str) {
    self.top_level_idents.insert(ident.to_string());
  }

  pub fn extend_used_idents(&mut self, idents: HashSet<String>) {
    self.top_level_idents.extend(idents);
  }

  pub fn used_idents(&self) -> &HashSet<String> {
    &self.top_level_idents
  }

  pub fn generate(&mut self) -> String {
    let mut minified_ident = self.next();

    while self.top_level_idents.contains(&minified_ident) {
      minified_ident = self.next();
    }

    self.top_level_idents.insert(minified_ident.to_string());

    minified_ident
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_minified_idents_generator() {
    let mut minified_idents_generator = MinifiedIdentsGenerator::new(Default::default());

    assert_eq!(&minified_idents_generator.next(), "a");
    assert_eq!(&minified_idents_generator.next(), "b");

    let mut minified_idents_generator = MinifiedIdentsGenerator {
      current_index: 26 * 3,
      top_level_idents: HashSet::new(),
    };
    assert_eq!(&minified_idents_generator.next(), "ca");
    assert_eq!(&minified_idents_generator.next(), "cb");

    let mut minified_idents_generator =
      MinifiedIdentsGenerator::new(HashSet::from(["a".to_string()]));
    assert_eq!(&minified_idents_generator.generate(), "b");
  }
}
