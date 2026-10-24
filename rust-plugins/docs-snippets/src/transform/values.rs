use farmfe_core::swc_ecma_ast::*;

pub fn string(expression: &Expr) -> Option<String> {
  match expression {
    Expr::Lit(Lit::Str(value)) => value.value.as_str().map(str::to_owned),
    Expr::Paren(value) => string(&value.expr),
    Expr::Tpl(value) if value.exprs.is_empty() => Some(
      value
        .quasis
        .iter()
        .map(|quasi| quasi.cooked.as_ref()?.as_str().map(str::to_owned))
        .collect::<Option<Vec<_>>>()?
        .concat(),
    ),
    _ => None,
  }
}

pub fn attribute_string(attribute: &JSXAttr) -> Option<String> {
  match &attribute.value {
    Some(JSXAttrValue::Str(value)) => value.value.as_str().map(str::to_owned),
    Some(JSXAttrValue::JSXExprContainer(value)) => match &value.expr {
      JSXExpr::Expr(value) => string(value),
      _ => None,
    },
    _ => None,
  }
}

pub fn attribute_boolean(attribute: &JSXAttr) -> Option<bool> {
  match &attribute.value {
    None => Some(true),
    Some(JSXAttrValue::JSXExprContainer(container)) => match &container.expr {
      JSXExpr::Expr(value) => match value.as_ref() {
        Expr::Lit(Lit::Bool(value)) => Some(value.value),
        _ => None,
      },
      _ => None,
    },
    _ => None,
  }
}

// JSX text follows React's whitespace rules. String expressions remain exact.
fn jsx_text(input: &str) -> String {
  let lines = input.split(['\r', '\n']).collect::<Vec<_>>();
  let last_content = lines
    .iter()
    .rposition(|line| !line.trim_matches([' ', '\t']).is_empty());
  let mut output = String::new();
  for (index, line) in lines.iter().enumerate() {
    let line = line.replace('\t', " ");
    let line = if index != 0 {
      line.trim_start_matches(' ')
    } else {
      &line
    };
    let line = if index + 1 != lines.len() {
      line.trim_end_matches(' ')
    } else {
      line
    };
    if !line.is_empty() {
      output.push_str(line);
      if Some(index) != last_content {
        output.push(' ');
      }
    }
  }
  output
}

pub fn children(children: &[JSXElementChild]) -> Result<String, String> {
  let mut output = String::new();
  for child in children {
    match child {
      JSXElementChild::JSXText(value) => output.push_str(&jsx_text(&value.value)),
      JSXElementChild::JSXExprContainer(value) => match &value.expr {
        JSXExpr::JSXEmptyExpr(_) => {}
        JSXExpr::Expr(value) => {
          output.push_str(&string(value).ok_or("code must be a static UTF-8 string")?)
        }
      },
      _ => return Err("code cannot contain elements or spreads".into()),
    }
  }
  Ok(output)
}
