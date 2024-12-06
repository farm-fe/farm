use std::{collections::HashMap, fmt::Display, io::Write};

#[derive(Debug)]
pub struct TemplateParseError {
  message: String,
}

impl std::fmt::Display for TemplateParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Failed to parse template: {}", self.message)
  }
}

impl<T: AsRef<str>> From<T> for TemplateParseError {
  fn from(value: T) -> Self {
    Self {
      message: value.as_ref().to_string(),
    }
  }
}

impl std::error::Error for TemplateParseError {}

type Result<T> = std::result::Result<T, TemplateParseError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Token<'a> {
  OBracket,
  CBracket,
  Bang,
  If,
  Var(&'a [u8]),
  Else,
  EndIf,
  Text(&'a [u8]),
  Invalid(usize, char),
}

impl Display for Token<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Token::OBracket => write!(f, "{{%"),
      Token::CBracket => write!(f, "%}}"),
      Token::Bang => write!(f, "!"),
      Token::If => write!(f, "if"),
      Token::Var(var) => write!(f, "{} (variable)", String::from_utf8_lossy(var)),
      Token::Else => write!(f, "else"),
      Token::EndIf => write!(f, "endif"),
      Token::Text(_) => write!(f, "(text)"),
      Token::Invalid(col, token) => {
        write!(f, "invalid token {token} at {col}",)
      }
    }
  }
}

const TRUE: &[u8] = b"true";
const FALSE: &[u8] = b"false";
const KEYWORDS: &[(&[u8], Token)] = &[
  (b"if", Token::If),
  (b"else", Token::Else),
  (b"endif", Token::EndIf),
];

struct Lexer<'a> {
  bytes: &'a [u8],
  len: usize,
  cursor: usize,
  in_bracket: bool,
}

impl<'a> Lexer<'a> {
  fn new(bytes: &'a [u8]) -> Self {
    let len = bytes.len();
    Self {
      len,
      bytes,
      cursor: 0,
      in_bracket: false,
    }
  }

  fn current_char(&self) -> char {
    self.bytes[self.cursor] as char
  }

  fn next_char(&self) -> char {
    self.bytes[self.cursor + 1] as char
  }

  fn skip_whitespace(&mut self) {
    while self.cursor < self.len && self.current_char().is_whitespace() {
      self.cursor += 1;
    }
  }

  fn is_symbol_start(&self) -> bool {
    let c = self.current_char();
    c.is_alphabetic() || c == '_'
  }

  fn is_symbol(&self) -> bool {
    let c = self.current_char();
    c.is_alphanumeric() || c == '_'
  }

  fn read_symbol(&mut self) -> &'a [u8] {
    let start = self.cursor;
    while self.is_symbol() {
      self.cursor += 1;
    }
    let end = self.cursor - 1;
    &self.bytes[start..=end]
  }

  fn next(&mut self) -> Option<Token<'a>> {
    if self.in_bracket {
      self.skip_whitespace();
    }

    if self.cursor >= self.len {
      return None;
    }

    if self.current_char() == '{' && self.next_char() == '%' {
      self.in_bracket = true;
      self.cursor += 2;
      return Some(Token::OBracket);
    }

    if self.current_char() == '%' && self.next_char() == '}' {
      self.in_bracket = false;
      self.cursor += 2;
      return Some(Token::CBracket);
    }

    if self.current_char() == '!' {
      self.cursor += 1;
      return Some(Token::Bang);
    }

    if self.in_bracket {
      if self.is_symbol_start() {
        let symbol = self.read_symbol();
        for (keyword, t) in KEYWORDS {
          if *keyword == symbol {
            return Some(*t);
          }
        }

        return Some(Token::Var(symbol));
      } else {
        self.cursor += 1;
        return Some(Token::Invalid(self.cursor, self.current_char()));
      }
    }

    if !self.in_bracket {
      let start = self.cursor;
      while !(self.current_char() == '{' && self.next_char() == '%') {
        self.cursor += 1;

        if self.cursor >= self.len {
          break;
        }
      }
      let end = self.cursor - 1;
      return Some(Token::Text(&self.bytes[start..=end]));
    }

    None
  }
}

impl<'a> Iterator for Lexer<'a> {
  type Item = Token<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    self.next()
  }
}

fn is_truthy(value: &[u8]) -> bool {
  match value {
    TRUE => true,
    FALSE => false,
    _ => !value.is_empty(),
  }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Stmt<'a> {
  Text(&'a [u8]),
  Var(&'a [u8]),
  If {
    var: &'a [u8],
    negated: bool,
    condition: Vec<Stmt<'a>>,
    else_condition: Option<Vec<Stmt<'a>>>,
  },
}

impl<'a> Stmt<'a> {
  fn execute<V, T>(&self, out: &mut T, data: &HashMap<&str, V>) -> Result<()>
  where
    T: Write,
    V: AsRef<[u8]>,
  {
    match self {
      Stmt::Text(t) => {
        out.write_all(t).map_err(|e| e.to_string())?;
      }
      Stmt::Var(var) => {
        let var = std::str::from_utf8(var).map_err(|e| e.to_string())?;
        let value = data
          .get(var)
          .ok_or_else(|| format!("Unrecognized variable: {var}"))?;
        out.write_all(value.as_ref()).map_err(|e| e.to_string())?;
      }
      Stmt::If {
        var,
        negated,
        condition,
        else_condition,
      } => {
        let var = std::str::from_utf8(var).map_err(|e| e.to_string())?;
        let value = data
          .get(var)
          .ok_or_else(|| format!("Unrecognized variable: {var}"))?;
        let value = value.as_ref();

        let truthy = is_truthy(value);
        let evaluated = if (truthy && !negated) || (!truthy && *negated) {
          condition
        } else if let Some(else_condition) = else_condition {
          else_condition
        } else {
          // no need to do anything, return early
          return Ok(());
        };

        for stmt in evaluated {
          stmt.execute(out, data)?;
        }
      }
    }

    Ok(())
  }
}

struct Parser<'a> {
  tokens: &'a [Token<'a>],
  len: usize,
  cursor: usize,
}

impl<'a> Parser<'a> {
  fn new(tokens: &'a [Token<'a>]) -> Self {
    Self {
      len: tokens.len(),
      tokens,
      cursor: 0,
    }
  }

  fn current_token(&self) -> Token<'a> {
    self.tokens[self.cursor]
  }

  fn skip_brackets(&mut self) {
    if self.cursor < self.len {
      while self.current_token() == Token::OBracket || self.current_token() == Token::CBracket {
        self.cursor += 1;

        if self.cursor >= self.len {
          break;
        }
      }
    }
  }

  fn consume_text(&mut self) -> Option<&'a [u8]> {
    if let Token::Text(text) = self.current_token() {
      self.cursor += 1;
      Some(text)
    } else {
      None
    }
  }

  fn consume_var(&mut self) -> Option<&'a [u8]> {
    if let Token::Var(var) = self.current_token() {
      self.cursor += 1;
      Some(var)
    } else {
      None
    }
  }

  fn consume_if(&mut self) -> Result<Option<Stmt<'a>>> {
    if self.current_token() == Token::If {
      self.cursor += 1;

      let negated = if self.current_token() == Token::Bang {
        self.cursor += 1;
        true
      } else {
        false
      };

      let var = self.consume_var().ok_or_else(|| {
        format!(
          "expected variable after if, found: {}",
          self.current_token()
        )
      })?;

      let mut condition = Vec::new();
      while self.current_token() != Token::Else || self.current_token() != Token::EndIf {
        match self.next()? {
          Some(stmt) => condition.push(stmt),
          None => break,
        }
      }

      let else_condition = if self.current_token() == Token::Else {
        self.cursor += 1;

        let mut else_condition = Vec::new();
        while self.current_token() != Token::EndIf {
          match self.next()? {
            Some(stmt) => else_condition.push(stmt),
            None => break,
          }
        }

        Some(else_condition)
      } else {
        None
      };

      if self.current_token() == Token::EndIf {
        self.cursor += 1;
      } else {
        return Err(format!("expected endif, found: {}", self.current_token()).into());
      }

      Ok(Some(Stmt::If {
        var,
        negated,
        condition,
        else_condition,
      }))
    } else {
      Ok(None)
    }
  }

  fn next(&mut self) -> Result<Option<Stmt<'a>>> {
    self.skip_brackets();

    if self.cursor >= self.len {
      return Ok(None);
    }

    if let t @ Token::Invalid(_, _) = self.current_token() {
      return Err(t.to_string().into());
    }

    let text = self.consume_text();
    if text.is_some() {
      return Ok(text.map(Stmt::Text));
    }

    let var = self.consume_var();
    if var.is_some() {
      return Ok(var.map(Stmt::Var));
    }

    let if_ = self.consume_if()?;
    if if_.is_some() {
      return Ok(if_);
    }

    Ok(None)
  }
}

pub fn render<T, V>(template: T, data: &HashMap<&str, V>) -> Result<String>
where
  T: AsRef<[u8]>,
  V: AsRef<[u8]>,
{
  let template = template.as_ref();
  let tokens: Vec<Token> = Lexer::new(template).collect();
  let mut parser = Parser::new(&tokens);
  let mut stmts: Vec<Stmt> = Vec::new();
  while let Some(stmt) = parser.next()? {
    stmts.push(stmt);
  }
  let mut out = Vec::new();
  for stmt in stmts {
    stmt.execute(&mut out, data)?;
  }

  String::from_utf8(out).map_err(|e| e.to_string().into())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn it_replaces_variable() {
    let template = "<html>Hello {% name %}</html>";
    let data: HashMap<&str, &str> = [("name", "world")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello world</html>")
  }

  #[test]
  fn it_replaces_variable_with_new_lines() {
    let template = r#"
        <html>
        <h1>Hello<h2>
        <em>{% name %}</em>
        </html>"#;
    let data: HashMap<&str, &str> = [("name", "world")].into();
    let rendered = render(template, &data).expect("it should render");
    let expected = r#"
        <html>
        <h1>Hello<h2>
        <em>world</em>
        </html>"#;
    assert_eq!(rendered, expected)
  }

  #[test]
  fn it_performs_condition() {
    let template = "<html>Hello {% if alpha %}alpha{% else %}stable{% endif %}</html>";
    let data: HashMap<&str, &str> = [("alpha", "true")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello alpha</html>")
  }

  #[test]
  fn it_performs_else_condition() {
    let template = "<html>Hello {% if alpha %}alpha{% else %}stable{% endif %}</html>";
    let data: HashMap<&str, &str> = [("alpha", "false")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello stable</html>")
  }

  #[test]
  fn it_performs_condition_with_new_lines() {
    let template = r#"
        <html>
        <h1>Hello<h2>{% if alpha %}
        <em>alpha</em>{% else %}
        <em>stable</em>{% endif %}
        </html>"#;
    let data: HashMap<&str, &str> = [("alpha", "true")].into();
    let rendered = render(template, &data).expect("it should render");
    let expected = r#"
        <html>
        <h1>Hello<h2>
        <em>alpha</em>
        </html>"#;
    assert_eq!(rendered, expected)
  }

  #[test]
  fn it_replaces_variable_within_if() {
    let template = r#"
        <html>
        <h1>Hello<h2>{% if alpha %}
        <em>{% alpha_str %}</em>{% else %}
        <em>stable</em>{% endif %}
        </html>"#;
    let data: HashMap<&str, &str> = [("alpha", "true"), ("alpha_str", "hello alpha")].into();
    let rendered = render(template, &data).expect("it should render");
    let expected = r#"
        <html>
        <h1>Hello<h2>
        <em>hello alpha</em>
        </html>"#;
    assert_eq!(rendered, expected)
  }

  #[test]
  fn it_performs_nested_conditions() {
    let template = r#"
        <html>
        <h1>Hello<h2>{% if alpha %}
        <em>{% alpha_str %}</em>{% else %}
        <em>{% if beta %}beta{%else%}stable{%endif%}</em>{% endif %}
        </html>"#;
    let data: HashMap<&str, &str> = [
      ("alpha", "false"),
      ("beta", "true"),
      ("alpha_str", "hello alpha"),
    ]
    .into();
    let rendered = render(template, &data).expect("it should render");
    let expected = r#"
        <html>
        <h1>Hello<h2>
        <em>beta</em>
        </html>"#;
    assert_eq!(rendered, expected)
  }

  #[test]
  fn truthy_and_falsy() {
    let template = "<html>Hello {% if beforeDevCommand %}{% beforeDevCommand %}{% endif %}</html>";
    let data: HashMap<&str, &str> = [("beforeDevCommand", "pnpm run")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello pnpm run</html>");

    let template = "<html>Hello {% if beforeDevCommand %}{% beforeDevCommand %}{% endif %}</html>";
    let data: HashMap<&str, &str> = [("beforeDevCommand", "")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello </html>");
  }

  #[test]
  fn negated_value() {
    let template = "<html>Hello {% if !name %}world{% else %}{ %name% }{%endif %}</html>";
    let data: HashMap<&str, &str> = [("name", "")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello world</html>");

    let template = "<html>Hello {% if !name %}world{% else %}{% name %}{%endif %}</html>";
    let data: HashMap<&str, &str> = [("name", "farm")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello farm</html>");

    let template = "<html>Hello {% if !render %}world{% else %}{% name %}{%endif %}</html>";
    let data: HashMap<&str, &str> = [("render", "true"), ("name", "farm")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello farm</html>");

    let template = "<html>Hello {% if !render %}world{% else %}{% name %}{%endif %}</html>";
    let data: HashMap<&str, &str> = [("render", "false"), ("name", "farm")].into();
    let rendered = render(template, &data).expect("it should render");
    assert_eq!(rendered, "<html>Hello world</html>");
  }

  #[test]
  #[should_panic]
  fn it_panics() {
    let template = "<html>Hello {% name }</html>";
    let data: HashMap<&str, &str> = [("name", "world")].into();
    render(template, &data).unwrap();
  }
}
