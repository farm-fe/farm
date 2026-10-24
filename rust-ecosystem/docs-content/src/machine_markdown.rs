//! Conservative, semantic Markdown projection of the exact retained HTML/MDX tree.
use crate::ParsedContent;
use mdxjs::hast::{self, Node, PropertyValue};
use std::collections::HashMap;
use swc_core::ecma::{
  ast::{
    Expr, ImportSpecifier, JSXAttr, JSXAttrName, JSXAttrOrSpread, JSXAttrValue, JSXElement,
    JSXElementChild, JSXElementName, JSXExpr, JSXExprContainer, Lit, ModuleDecl, ModuleExportName,
    ModuleItem, Prop, PropName, PropOrSpread,
  },
  visit::{Visit, VisitWith},
};

fn unsupported(detail: &str) -> String {
  format!(
    "[DOCS_AI_UNSUPPORTED] {detail}; provide static Markdown or a supported Farm Docs component"
  )
}
fn escaped(value: &str) -> String {
  let mut result = String::new();
  for ch in value.chars() {
    if matches!(
      ch,
      '\\' | '`' | '*' | '_' | '[' | ']' | '<' | '>' | '#' | '|' | '~'
    ) {
      result.push('\\');
    }
    result.push(ch);
  }
  result
}
fn property<'a>(properties: &'a [(String, PropertyValue)], name: &str) -> Option<&'a str> {
  properties.iter().find_map(|(key, value)| match value {
    PropertyValue::String(value) if key == name => Some(value.as_str()),
    _ => None,
  })
}
fn table_cell(value: &str) -> String {
  let mut output = String::new();
  let mut backslashes = 0;
  for character in value.trim().chars() {
    if character == '|' && backslashes % 2 == 0 {
      output.push('\\');
    }
    if character == '\n' {
      output.push_str("<br>");
    } else {
      output.push(character);
    }
    backslashes = if character == '\\' {
      backslashes + 1
    } else {
      0
    };
  }
  output
}
fn text(node: &Node) -> String {
  match node {
    Node::Text(value) => value.value.clone(),
    _ => node
      .children()
      .map(|children| children.iter().map(text).collect())
      .unwrap_or_default(),
  }
}
// Accept only the static color-token shape emitted by the native inline compiler.
// Forged dynamic components, spread attributes and executable style expressions fail.
fn highlighted_inline_text(children: &[Node]) -> Result<String, String> {
  let mut output = String::new();
  for child in children {
    let Node::MdxJsxElement(element) = child else {
      return Err(unsupported("invalid highlighted inline token"));
    };
    if element.name.as_deref() != Some("span") || element.attributes.len() != 2 {
      return Err(unsupported("invalid highlighted inline span"));
    }
    let mut class = false;
    let mut style = false;
    for attribute in &element.attributes {
      let hast::AttributeContent::Property(attribute) = attribute else {
        return Err(unsupported("spread inline attributes"));
      };
      match (attribute.name.as_str(), &attribute.value) {
        ("className", Some(hast::AttributeValue::Literal(value)))
          if value == "farm-code-token" && !class =>
        {
          class = true
        }
        ("style", Some(hast::AttributeValue::Expression(value))) if !style => {
          let values: serde_json::Value =
            serde_json::from_str(&value.value).map_err(|_| unsupported("dynamic inline style"))?;
          let Some(values) = values.as_object() else {
            return Err(unsupported("invalid inline style"));
          };
          if values.len() != 2
            || !["--code-light", "--code-dark"].iter().all(|name| {
              values
                .get(*name)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|color| {
                  color.len() == 7
                    && color.starts_with('#')
                    && color.as_bytes()[1..].iter().all(u8::is_ascii_hexdigit)
                })
            })
          {
            return Err(unsupported("invalid inline palette"));
          }
          style = true;
        }
        _ => return Err(unsupported("unsupported inline attribute")),
      }
    }
    if !class || !style {
      return Err(unsupported("incomplete inline palette"));
    }
    for child in &element.children {
      let Node::Text(value) = child else {
        return Err(unsupported("dynamic inline text"));
      };
      output.push_str(&value.value);
    }
  }
  Ok(output)
}
fn fence(code: &str, language: &str) -> Result<String, String> {
  if language
    .chars()
    .any(|ch| ch.is_control() || ch.is_whitespace() || matches!(ch, '`' | '~'))
  {
    return Err(unsupported("invalid code language"));
  }
  let length = code
    .split(|ch| ch != '`')
    .map(str::len)
    .max()
    .unwrap_or(0)
    .max(2)
    + 1;
  let marker = "`".repeat(length);
  Ok(format!(
    "{marker}{language}\n{code}{}{marker}\n\n",
    if code.ends_with('\n') { "" } else { "\n" }
  ))
}
fn destination(value: &str) -> Result<String, String> {
  let lower = value.trim_start().to_ascii_lowercase();
  if value.contains(['\n', '\r', '\\'])
    || value.chars().any(char::is_control)
    || lower.starts_with("file:")
    || lower.starts_with("javascript:")
    || lower.starts_with("data:")
    || (value.as_bytes().get(1) == Some(&b':') && !lower.starts_with("https:"))
  {
    return Err(unsupported("unsafe or local content URL"));
  }
  Ok(
    value
      .replace(' ', "%20")
      .replace('<', "%3C")
      .replace('>', "%3E")
      .replace('(', "%28")
      .replace(')', "%29"),
  )
}

struct StaticExpressions(HashMap<u32, Option<String>>);
impl Visit for StaticExpressions {
  fn visit_jsx_expr_container(&mut self, expression: &JSXExprContainer) {
    let value = match &expression.expr {
      JSXExpr::JSXEmptyExpr(_) => Some(String::new()),
      JSXExpr::Expr(value) => match value.as_ref() {
        Expr::Lit(Lit::Str(value)) => Some(value.value.to_string()),
        Expr::Tpl(value) if value.exprs.is_empty() => value
          .quasis
          .first()
          .and_then(|part| part.cooked.as_ref())
          .map(ToString::to_string),
        _ => None,
      },
    };
    self.0.insert(expression.span.lo.0, value);
    expression.visit_children_with(self);
  }
}
struct Writer {
  components: HashMap<String, String>,
  expressions: HashMap<u32, Option<String>>,
  static_components: HashMap<u32, Result<String, String>>,
  static_styles: HashMap<u32, bool>,
}

fn static_style(attribute: &JSXAttr) -> bool {
  let Some(JSXAttrValue::JSXExprContainer(container)) = &attribute.value else {
    return false;
  };
  let JSXExpr::Expr(expression) = &container.expr else {
    return false;
  };
  let Expr::Object(object) = expression.as_ref() else {
    return false;
  };
  object.props.iter().all(|property| {
    let PropOrSpread::Prop(property) = property else {
      return false;
    };
    let Prop::KeyValue(property) = property.as_ref() else {
      return false;
    };
    let name = match &property.key {
      PropName::Ident(name) => name.sym.as_ref(),
      PropName::Str(name) => name.value.as_ref(),
      _ => return false,
    };
    if name.is_empty() || matches!(name, "__proto__" | "prototype" | "constructor") {
      return false;
    }
    match property.value.as_ref() {
      Expr::Lit(Lit::Str(_) | Lit::Bool(_) | Lit::Null(_)) => true,
      Expr::Lit(Lit::Num(value)) => value.value.is_finite(),
      _ => false,
    }
  })
}
struct StaticStyles(HashMap<u32, bool>);
impl Visit for StaticStyles {
  fn visit_jsx_element(&mut self, element: &JSXElement) {
    let styles: Vec<_> = element.opening.attrs.iter().filter_map(|attribute| match attribute {
      JSXAttrOrSpread::JSXAttr(attribute) if matches!(&attribute.name, JSXAttrName::Ident(name) if name.sym == "style") => Some(attribute),
      _ => None,
    }).collect();
    if !styles.is_empty() {
      self.0.insert(
        element.span.lo.0,
        styles.len() == 1 && static_style(styles[0]),
      );
    }
    element.visit_children_with(self);
  }
}

fn static_string(expression: &Expr) -> Option<String> {
  match expression {
    Expr::Lit(Lit::Str(value)) => Some(value.value.to_string()),
    Expr::Paren(value) => static_string(&value.expr),
    Expr::Tpl(value) if value.exprs.is_empty() => value
      .quasis
      .iter()
      .map(|part| part.cooked.as_ref().map(ToString::to_string))
      .collect::<Option<Vec<_>>>()
      .map(|parts| parts.concat()),
    _ => None,
  }
}
fn attribute_string(attribute: &JSXAttr) -> Option<String> {
  match &attribute.value {
    Some(JSXAttrValue::Lit(Lit::Str(value))) => Some(value.value.to_string()),
    Some(JSXAttrValue::JSXExprContainer(value)) => match &value.expr {
      JSXExpr::Expr(expression) => static_string(expression),
      _ => None,
    },
    _ => None,
  }
}
// Match the snippets compiler's React JSX-text whitespace rules. String
// expression children remain exact and are never evaluated.
fn jsx_text(input: &str) -> String {
  let lines: Vec<_> = input.split(['\r', '\n']).collect();
  let last = lines
    .iter()
    .rposition(|line| !line.trim_matches([' ', '\t']).is_empty());
  let mut output = String::new();
  for (index, line) in lines.iter().enumerate() {
    let line = line.replace('\t', " ");
    let line = if index == 0 {
      &line
    } else {
      line.trim_start_matches(' ')
    };
    let line = if index + 1 == lines.len() {
      line
    } else {
      line.trim_end_matches(' ')
    };
    if !line.is_empty() {
      output.push_str(line);
      if Some(index) != last {
        output.push(' ');
      }
    }
  }
  output
}
fn code_children(children: &[JSXElementChild]) -> Result<String, String> {
  let mut output = String::new();
  for child in children {
    match child {
      JSXElementChild::JSXText(value) => output.push_str(&jsx_text(&value.value)),
      JSXElementChild::JSXExprContainer(value) => match &value.expr {
        JSXExpr::JSXEmptyExpr(_) => {}
        JSXExpr::Expr(expression) => output.push_str(
          &static_string(expression).ok_or_else(|| unsupported("dynamic CodeBlock children"))?,
        ),
      },
      _ => {
        return Err(unsupported(
          "CodeBlock children contain elements or spreads",
        ));
      }
    }
  }
  Ok(output)
}
fn static_codeblock(element: &JSXElement) -> Result<String, String> {
  let mut attributes = HashMap::new();
  for attribute in &element.opening.attrs {
    let JSXAttrOrSpread::JSXAttr(attribute) = attribute else {
      return Err(unsupported("CodeBlock spread attributes"));
    };
    let JSXAttrName::Ident(name) = &attribute.name else {
      return Err(unsupported("CodeBlock namespaced attributes"));
    };
    let name = name.sym.as_ref();
    if ![
      "code",
      "language",
      "title",
      "showLineNumbers",
      "highlightLines",
    ]
    .contains(&name)
    {
      return Err(unsupported("unknown CodeBlock attribute"));
    }
    let value = if name == "showLineNumbers" {
      match &attribute.value {
        None => String::new(),
        Some(JSXAttrValue::JSXExprContainer(value)) if matches!(&value.expr, JSXExpr::Expr(value) if matches!(value.as_ref(), Expr::Lit(Lit::Bool(_)))) => {
          String::new()
        }
        _ => return Err(unsupported("dynamic showLineNumbers")),
      }
    } else {
      attribute_string(attribute)
        .ok_or_else(|| unsupported("CodeBlock attributes must be static strings"))?
    };
    if attributes.insert(name.to_string(), value).is_some() {
      return Err(unsupported("duplicate CodeBlock attribute"));
    }
  }
  let children = code_children(&element.children)?;
  if attributes.contains_key("code") && !children.is_empty() {
    return Err(unsupported("use CodeBlock code or children, not both"));
  }
  let code = attributes.get("code").unwrap_or(&children);
  if let Some(ranges) = attributes.get("highlightLines") {
    farmfe_syntax_highlight::parse_line_ranges(ranges, code.lines().count().max(1))
      .map_err(|_| unsupported("invalid code highlight range"))?;
  }
  let title = attributes
    .get("title")
    .map(|value| format!("**{}**\n\n", escaped(value)))
    .unwrap_or_default();
  Ok(format!(
    "{title}{}",
    fence(
      &format!("{code}\n"),
      attributes
        .get("language")
        .map(String::as_str)
        .unwrap_or("text")
    )?
  ))
}
fn static_commands(element: &JSXElement) -> Result<String, String> {
  if element.opening.attrs.len() != 1 || !code_children(&element.children)?.trim().is_empty() {
    return Err(unsupported(
      "CommandTabs requires one static commands attribute and no children",
    ));
  }
  let JSXAttrOrSpread::JSXAttr(attribute) = &element.opening.attrs[0] else {
    return Err(unsupported("CommandTabs spread attributes"));
  };
  if !matches!(&attribute.name, JSXAttrName::Ident(name) if name.sym == "commands") {
    return Err(unsupported("CommandTabs requires commands"));
  }
  let Some(JSXAttrValue::JSXExprContainer(container)) = &attribute.value else {
    return Err(unsupported("CommandTabs commands must be a static object"));
  };
  let JSXExpr::Expr(expression) = &container.expr else {
    return Err(unsupported("CommandTabs expression"));
  };
  let Expr::Object(object) = expression.as_ref() else {
    return Err(unsupported("dynamic CommandTabs mapping"));
  };
  let mut commands = HashMap::new();
  for property in &object.props {
    let PropOrSpread::Prop(property) = property else {
      return Err(unsupported("CommandTabs spread mapping"));
    };
    let Prop::KeyValue(property) = property.as_ref() else {
      return Err(unsupported("CommandTabs mapping property"));
    };
    let tool = match &property.key {
      PropName::Ident(value) => value.sym.to_string(),
      PropName::Str(value) => value.value.to_string(),
      _ => return Err(unsupported("computed CommandTabs tool")),
    };
    if !["npm", "pnpm", "yarn", "bun"].contains(&tool.as_str()) {
      return Err(unsupported("unknown CommandTabs tool"));
    }
    let value =
      static_string(&property.value).ok_or_else(|| unsupported("dynamic CommandTabs command"))?;
    if value.trim().is_empty() || commands.insert(tool, value).is_some() {
      return Err(unsupported("empty or duplicate CommandTabs command"));
    }
  }
  if commands.is_empty() {
    return Err(unsupported("empty CommandTabs mapping"));
  }
  let mut output = String::new();
  for tool in ["npm", "pnpm", "yarn", "bun"] {
    if let Some(value) = commands.get(tool) {
      output.push_str(&format!(
        "**{tool}**\n\n{}",
        fence(&format!("{value}\n"), "sh")?
      ));
    }
  }
  Ok(output)
}
struct StaticComponents<'a> {
  names: &'a HashMap<String, String>,
  values: HashMap<u32, Result<String, String>>,
}
impl Visit for StaticComponents<'_> {
  fn visit_jsx_element(&mut self, element: &JSXElement) {
    if let JSXElementName::Ident(name) = &element.opening.name {
      if let Some(component) = self.names.get(name.sym.as_ref()) {
        let value = match component.as_str() {
          "CodeBlock" => Some(static_codeblock(element)),
          "CommandTabs" => Some(static_commands(element)),
          _ => None,
        };
        if let Some(value) = value {
          self.values.insert(element.span.lo.0, value);
        }
      }
    }
    element.visit_children_with(self);
  }
}
impl Writer {
  fn children(&self, nodes: &[Node]) -> Result<String, String> {
    nodes.iter().map(|node| self.node(node)).collect()
  }
  fn literal(&self, node: &Node) -> Result<String, String> {
    if let Node::Text(value) = node {
      return Ok(value.value.clone());
    }
    if let Node::MdxExpression(_) = node {
      let key = node
        .position()
        .map(|position| position.start.offset as u32 + 1);
      return key
        .and_then(|key| self.expressions.get(&key))
        .and_then(Clone::clone)
        .ok_or_else(|| unsupported("dynamic MDX expression"));
    }
    Err(unsupported(
      "code children must be literal text or a static string/template",
    ))
  }
  fn attribute(element: &hast::MdxJsxElement, name: &str) -> Result<Option<String>, String> {
    for attribute in &element.attributes {
      match attribute {
        hast::AttributeContent::Property(value) if value.name == name => {
          return match &value.value {
            Some(hast::AttributeValue::Literal(value)) => Ok(Some(value.clone())),
            _ => Err(unsupported(&format!("dynamic {name} attribute"))),
          };
        }
        hast::AttributeContent::Expression(_) => return Err(unsupported("spread JSX attributes")),
        _ => {}
      }
    }
    Ok(None)
  }
  fn jsx(&self, element: &hast::MdxJsxElement) -> Result<String, String> {
    let Some(name) = element.name.as_deref() else {
      return self.children(&element.children);
    };
    // Highlighted Markdown fences are retained as HAST Elements, not author
    // JSX. Do not trust author JSX to carry the compiler's private code text.
    if name == "pre" {
      return Err(unsupported(
        "raw MDX pre; use a Markdown fence or Farm Docs CodeBlock",
      ));
    }
    let component = self.components.get(name).map(String::as_str).unwrap_or("");
    if matches!(component, "Tabs" | "TabItem") {
      for attribute in &element.attributes {
        let hast::AttributeContent::Property(attribute) = attribute else {
          return Err(unsupported("spread tab attributes"));
        };
        if !matches!(
          attribute.name.as_str(),
          "value" | "label" | "groupId" | "defaultValue" | "className" | "default"
        ) {
          return Err(unsupported("unknown tab attribute"));
        }
        if attribute.name != "default"
          && !matches!(attribute.value, Some(hast::AttributeValue::Literal(_)))
        {
          return Err(unsupported("dynamic tab attribute"));
        }
        if attribute.name == "default" && attribute.value.is_some() {
          return Err(unsupported("dynamic tab default"));
        }
      }
    }
    match component {
      "Tabs" => self.children(&element.children),
      "TabItem" => {
        let label = Self::attribute(element, "label")?
          .or(Self::attribute(element, "value")?)
          .ok_or_else(|| unsupported("TabItem requires a static label or value"))?;
        Ok(format!(
          "**{}**\n\n{}\n",
          escaped(&label),
          self.children(&element.children)?
        ))
      }
      "CodeBlock" | "CommandTabs" => self
        .static_components
        .get(
          &(element
            .position
            .as_ref()
            .ok_or_else(|| unsupported("unpositioned CodeBlock"))?
            .start
            .offset as u32
            + 1),
        )
        .cloned()
        .ok_or_else(|| unsupported("uncompiled static CodeBlock"))?,
      _ if name.chars().next().is_some_and(char::is_lowercase) => {
        let mut properties = vec![];
        for attribute in &element.attributes {
          match attribute {
            hast::AttributeContent::Property(attribute) => {
              if attribute.name == "style"
                && matches!(attribute.value, Some(hast::AttributeValue::Expression(_)))
              {
                let key = element
                  .position
                  .as_ref()
                  .map(|position| position.start.offset as u32 + 1);
                if key.and_then(|key| self.static_styles.get(&key)).copied() == Some(true) {
                  // A proven literal CSS object affects presentation only. The
                  // semantic image/link/text attributes are still processed.
                  continue;
                }
                return Err(unsupported("dynamic or unsupported style object"));
              }
              let Some(hast::AttributeValue::Literal(value)) = &attribute.value else {
                return Err(unsupported("dynamic HTML attributes"));
              };
              if attribute.name.starts_with("on") {
                return Err(unsupported("interactive HTML handler"));
              }
              properties.push((attribute.name.clone(), PropertyValue::String(value.clone())));
            }
            _ => return Err(unsupported("spread HTML attributes")),
          }
        }
        self.element(name, &properties, &element.children)
      }
      _ => Err(unsupported(&format!("MDX component {name}"))),
    }
  }
  fn element(
    &self,
    name: &str,
    properties: &[(String, PropertyValue)],
    children: &[Node],
  ) -> Result<String, String> {
    if name == "pre" {
      let code = property(properties, "data-code")
        .map(str::to_owned)
        .unwrap_or_else(|| children.iter().map(text).collect());
      let language = property(properties, "data-language").unwrap_or("text");
      return fence(&code, language);
    }
    if name == "code" && property(properties, "className") == Some("farm-inline-code") {
      let value = highlighted_inline_text(children)?;
      let marker = "`".repeat(value.split(|ch| ch != '`').map(str::len).max().unwrap_or(0) + 1);
      return Ok(format!("{marker} {value} {marker}"));
    }
    let content = self.children(children)?;
    match name {
      "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
        let depth = name.as_bytes()[1] - b'0';
        let anchor = property(properties, "id")
          .map(|id| {
            format!(
              "<a id=\"{}\"></a>\n",
              id.replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;")
                .replace('>', "&gt;")
            )
          })
          .unwrap_or_default();
        Ok(format!(
          "{anchor}{} {content}\n\n",
          "#".repeat(depth as usize)
        ))
      }
      "p" => Ok(format!("{}\n\n", content.trim())),
      "strong" | "b" => Ok(format!("**{content}**")),
      "em" | "i" => Ok(format!("*{content}*")),
      "del" | "s" | "strike" => Ok(format!("~~{content}~~")),
      "code" => {
        let value: String = children.iter().map(text).collect();
        let marker = "`".repeat(value.split(|ch| ch != '`').map(str::len).max().unwrap_or(0) + 1);
        Ok(format!("{marker} {value} {marker}"))
      }
      "a" => match property(properties, "href") {
        Some(href) => Ok(format!("[{content}](<{}>)", destination(href)?)),
        None => Ok(content),
      },
      "img" => Ok(format!(
        "![{}](<{}>)",
        escaped(property(properties, "alt").unwrap_or("")),
        destination(property(properties, "src").ok_or_else(|| unsupported("image without src"))?)?
      )),
      "br" => Ok("  \n".into()),
      "hr" => Ok("\n---\n\n".into()),
      "ul" | "ol" => {
        let start = property(properties, "start")
          .and_then(|value| value.parse::<usize>().ok())
          .unwrap_or(1);
        let mut output = String::new();
        let mut index = 0;
        for child in children {
          if matches!(child, Node::Text(value) if value.value.trim().is_empty()) {
            continue;
          }
          let item = self.node(child)?;
          let marker = if name == "ol" {
            format!("{}. ", start + index)
          } else {
            "- ".into()
          };
          let mut lines = item.trim().lines();
          if let Some(first) = lines.next() {
            output.push_str(&format!("{marker}{first}\n"));
          }
          for line in lines {
            output.push_str(&format!("{}{}\n", " ".repeat(marker.len()), line));
          }
          index += 1;
        }
        Ok(format!("{output}\n"))
      }
      "li" => Ok(content),
      "blockquote" | "aside" => {
        let kind = property(properties, "data-type")
          .map(|kind| format!("> [!{}]\n", kind.to_uppercase()))
          .unwrap_or_default();
        Ok(format!(
          "{kind}{}\n\n",
          content
            .trim()
            .lines()
            .map(|line| format!("> {line}"))
            .collect::<Vec<_>>()
            .join("\n")
        ))
      }
      "table" => self.table(children),
      "thead" | "tbody" | "tfoot" | "tr" | "td" | "th" => Ok(content),
      "div" | "span" | "section" | "article" | "main" | "details" | "summary" => Ok(content),
      "input" if property(properties, "type") == Some("checkbox") => Ok(
        if properties
          .iter()
          .any(|(name, value)| name == "checked" && matches!(value, PropertyValue::Boolean(true)))
        {
          "[x] "
        } else {
          "[ ] "
        }
        .into(),
      ),
      _ => Err(unsupported(&format!("HTML element {name}"))),
    }
  }
  fn table(&self, nodes: &[Node]) -> Result<String, String> {
    fn rows<'a>(nodes: &'a [Node], result: &mut Vec<&'a [Node]>) {
      for node in nodes {
        if let Node::Element(element) = node {
          if element.tag_name == "tr" {
            result.push(&element.children);
          } else {
            rows(&element.children, result);
          }
        }
      }
    }
    let mut table = vec![];
    rows(nodes, &mut table);
    let mut output = String::new();
    for (index, row) in table.into_iter().enumerate() {
      let cells: Vec<_> = row
        .iter()
        .filter(|node| matches!(node, Node::Element(_)))
        .map(|node| self.node(node).map(|value| table_cell(&value)))
        .collect::<Result<_, _>>()?;
      output.push_str(&format!("| {} |\n", cells.join(" | ")));
      if index == 0 {
        output.push_str(&format!("| {} |\n", vec!["---"; cells.len()].join(" | ")));
      }
    }
    Ok(format!("{output}\n"))
  }
  fn node(&self, node: &Node) -> Result<String, String> {
    match node {
      Node::Root(root) => self.children(&root.children),
      Node::Text(value) => Ok(escaped(&value.value)),
      Node::Element(element) => {
        self.element(&element.tag_name, &element.properties, &element.children)
      }
      Node::MdxJsxElement(element) => self.jsx(element),
      Node::MdxExpression(_) => self.literal(node).map(|value| escaped(&value)),
      Node::MdxjsEsm(_) | Node::Comment(_) => Ok(String::new()),
      _ => Err(unsupported("document node")),
    }
  }
}

pub(super) fn export(parsed: &ParsedContent) -> Result<String, String> {
  let location = markdown::Location::new(parsed.prepared_source.as_bytes());
  let program = mdxjs::hast_util_to_swc(
    &parsed.tree,
    &parsed.options,
    Some(&location),
    &mut Default::default(),
  )
  .map_err(|_| unsupported("MDX syntax"))?;
  let mut components = HashMap::new();
  for item in &program.module.body {
    if let ModuleItem::ModuleDecl(declaration) = item {
      let ModuleDecl::Import(import) = declaration else {
        return Err(unsupported("MDX exports or executable declarations"));
      };
      let source = import.src.value.as_ref();
      if !matches!(source, "@farmfe/docs/mdx" | "@farmfe/docs/snippets") {
        continue;
      }
      for specifier in &import.specifiers {
        if let ImportSpecifier::Named(specifier) = specifier {
          let imported = match &specifier.imported {
            Some(ModuleExportName::Ident(value)) => value.sym.to_string(),
            Some(ModuleExportName::Str(value)) => value.value.to_string(),
            None => specifier.local.sym.to_string(),
          };
          if matches!(
            imported.as_str(),
            "Tabs" | "TabItem" | "CodeBlock" | "CommandTabs"
          ) {
            components.insert(specifier.local.sym.to_string(), imported);
          }
        }
      }
    }
  }
  let mut expressions = StaticExpressions(HashMap::new());
  program.module.visit_with(&mut expressions);
  let mut styles = StaticStyles(HashMap::new());
  program.module.visit_with(&mut styles);
  let static_components = {
    let mut collector = StaticComponents {
      names: &components,
      values: HashMap::new(),
    };
    program.module.visit_with(&mut collector);
    collector.values
  };
  let result = Writer {
    components,
    expressions: expressions.0,
    static_components,
    static_styles: styles.0,
  }
  .node(&parsed.tree)?;
  Ok(format!("{}\n", result.trim()))
}

#[cfg(test)]
mod tests {
  use super::*;
  fn convert(source: &str) -> Result<String, String> {
    crate::parse_content(
      source,
      &mdxjs::Options {
        filepath: Some("test.mdx".into()),
        ..Default::default()
      },
      Some(&Default::default()),
      crate::ContentFormat::Mdx,
    )?
    .machine_markdown()
  }
  #[test]
  fn preserves_fences_tables_admonitions_and_all_tabs() {
    let source = "import { Tabs, TabItem } from '@farmfe/docs/mdx';\n\n# Setup\n\n:::note Read first\nKeep **all** commands.\n:::\n\n<Tabs><TabItem value=\"npm\" label=\"npm\">\n\n```sh\nnpm run build\n```\n\n</TabItem><TabItem value=\"pnpm\" label=\"pnpm\">\n\n```sh\npnpm run build\n```\n\n</TabItem></Tabs>\n\n| A | B |\n| - | - |\n| one | two |\n";
    let result = convert(source).expect("supported content");
    assert!(result.contains("npm run build") && result.contains("pnpm run build"));
    assert!(result.contains("> [!NOTE]") && result.contains("| one | two |"));
  }
  #[test]
  fn rejects_unknown_components_and_dynamic_expressions() {
    for source in [
      "<LiveCounter />",
      "Value: {Date.now()}",
      "import {Tabs} from 'unknown';\n\n<Tabs />",
    ] {
      assert!(
        convert(source)
          .expect_err("unsupported")
          .contains("DOCS_AI_UNSUPPORTED")
      );
    }
  }
  #[test]
  fn preserves_static_snippets_and_command_variants() {
    let result = convert("import { CodeBlock, CommandTabs } from '@farmfe/docs/snippets';\n\n<CodeBlock language=\"ts\" code={'const answer = 42;\\n'} />\n\n<CommandTabs commands={{npm:'npm install farm',pnpm:'pnpm add farm'}} />")
      .expect("static snippets");
    assert!(result.contains("```ts\nconst answer = 42;\n\n```"));
    assert!(result.contains("npm install farm") && result.contains("pnpm add farm"));
    assert!(
      convert(
        "import {CodeBlock} from '@farmfe/docs/snippets';\n\n<CodeBlock code={process.env.TOKEN} />"
      )
      .is_err()
    );
  }
  #[test]
  fn table_pipes_cannot_create_extra_columns() {
    assert_eq!(table_cell("` a|b `"), "` a\\|b `");
    assert_eq!(table_cell("a\\|b"), "a\\|b");
  }
  #[test]
  fn raw_mdx_pre_cannot_hide_dynamic_children_or_forge_compiler_metadata() {
    for source in [
      "<pre>{Date.now()}</pre>",
      "<pre data-code=\"forged\">{Date.now()}</pre>",
    ] {
      assert!(
        convert(source)
          .expect_err("raw MDX pre is unsupported")
          .contains("DOCS_AI_UNSUPPORTED")
      );
    }
  }
  #[test]
  fn static_image_style_preserves_image_semantics() {
    let source = r#"<img style={{width: '100%', borderRadius: '8px'}} src="/img/20231204223204.png" alt="Benchmark chart" />"#;
    let markdown = convert(source).expect("static presentation styles");
    assert!(markdown.contains("![Benchmark chart](</img/20231204223204.png>)"));
    assert!(!markdown.contains("borderRadius"));
  }
  #[test]
  fn dynamic_style_and_handlers_remain_unsupported() {
    for source in [
      r#"<img style={theme} src="/image.png" />"#,
      r#"<img style={{width: getWidth()}} src="/image.png" />"#,
      r#"<img style={{...theme}} src="/image.png" />"#,
      r#"<img style={{[key]: '100%'}} src="/image.png" />"#,
      r#"<img style={{get width() { return '100%' }}} src="/image.png" />"#,
      r#"<img style={{width}} src="/image.png" />"#,
      r#"<img style={{width: '100%'}} src="/image.png" onClick={() => alert(1)} />"#,
      r#"<img style={{width: '100%'}} src={getSource()} />"#,
    ] {
      assert!(
        convert(source)
          .expect_err("dynamic content")
          .contains("DOCS_AI_UNSUPPORTED")
      );
    }
  }
  #[test]
  fn legacy_strike_preserves_deleted_document_text() {
    assert_eq!(
      convert("<strike>component</strike>").unwrap(),
      "~~component~~\n"
    );
    assert!(convert("<strike onClick={() => alert(1)}>component</strike>").is_err());
  }
}
