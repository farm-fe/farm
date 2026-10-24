//! Inspect the grammar scopes and current palette for the documented TS/TSX sample.
use farmfe_syntax_highlight::{HighlightOptions, highlight};
use two_face::re_exports::syntect::parsing::ParseState;
fn main() -> Result<(), Box<dyn std::error::Error>> {
  let source = "interface User { name: string; }\nclass Service<T> {\n  async load(id: number): Promise<void> {\n    const item: T | null = null; // comment\n    console.log(\"hello\", id);\n  }\n}\nconst view = <Button disabled={false}>Hello</Button>;\n";
  let syntax_set = two_face::syntax::extra_newlines();
  let syntax = syntax_set
    .find_syntax_by_token("tsx")
    .ok_or("TSX grammar missing")?;
  println!("SYNTAX {}", syntax.name);
  let mut parser = ParseState::new(syntax);
  for line in source.split_inclusive('\n') {
    println!("LINE {line:?}");
    for (offset, operation) in parser.parse_line(line, &syntax_set)? {
      println!("SCOPE {offset} {operation:?}");
    }
  }
  for line in highlight(source, "tsx", &HighlightOptions::default())?.lines {
    for token in line {
      println!("TOKEN {:?} {} {}", token.text, token.light, token.dark);
    }
  }
  Ok(())
}
