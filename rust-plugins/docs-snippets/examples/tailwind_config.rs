//! One-time static migration of website style data. Never executes JavaScript.
use farmfe_core::{
  module::ModuleId,
  swc_ecma_ast::*,
  swc_ecma_parser::{EsSyntax, Syntax},
};
use farmfe_toolkit::script::parse_module;
use serde_json::{Value, json};
use std::sync::Arc;

fn parse(source: &str) -> Result<Module, String> {
  Ok(
    parse_module(
      &ModuleId::from("style-migration.js"),
      Arc::new(source.to_owned()),
      Syntax::Es(EsSyntax::default()),
      EsVersion::latest(),
    )
    .map_err(|e| e.to_string())?
    .ast,
  )
}
fn name(key: &PropName) -> Result<String, String> {
  match key {
    PropName::Ident(v) => Ok(v.sym.to_string()),
    PropName::Str(v) => Ok(v.value.to_string_lossy().into_owned()),
    PropName::Num(v) => Ok(v.value.to_string()),
    _ => Err("Computed migration key".into()),
  }
}
fn literal(expr: &Expr) -> Result<Value, String> {
  match expr {
    Expr::Lit(Lit::Str(v)) => Ok(json!(v.value.to_string_lossy())),
    Expr::Lit(Lit::Bool(v)) => Ok(json!(v.value)),
    Expr::Lit(Lit::Num(v)) => serde_json::Number::from_f64(v.value)
      .map(Value::Number)
      .ok_or("Nonfinite number".into()),
    Expr::Lit(Lit::Null(_)) => Ok(Value::Null),
    Expr::Array(v) => Ok(Value::Array(
      v.elems
        .iter()
        .map(|e| {
          let e = e.as_ref().ok_or("Array hole")?;
          if e.spread.is_some() {
            return Err("Array spread".into());
          }
          literal(&e.expr)
        })
        .collect::<Result<_, String>>()?,
    )),
    Expr::Object(v) => {
      let mut out = serde_json::Map::new();
      for p in &v.props {
        let PropOrSpread::Prop(p) = p else {
          return Err("Object spread".into());
        };
        let Prop::KeyValue(p) = p.as_ref() else {
          return Err("Dynamic object property".into());
        };
        out.insert(name(&p.key)?, literal(&p.value)?);
      }
      Ok(Value::Object(out))
    }
    Expr::Paren(v) => literal(&v.expr),
    _ => Err("Only literal style data may be migrated".into()),
  }
}
fn palette(source: &str) -> Result<Value, String> {
  let ast = parse(source)?;
  let mut object = None;
  for item in &ast.body {
    if let ModuleItem::Stmt(Stmt::Decl(Decl::Var(decl))) = item {
      for v in &decl.decls {
        if matches!(&v.name,Pat::Ident(id) if id.id.sym=="_default") {
          if object.is_some() {
            return Err("Duplicate palette declaration".into());
          }
          let Some(expr) = &v.init else {
            return Err("Palette lacks value".into());
          };
          let Expr::Object(value) = expr.as_ref() else {
            return Err("Palette must be static object".into());
          };
          object = Some(value);
        }
      }
    }
  }
  let object = object.ok_or("Missing known palette declaration")?;
  let mut colors = serde_json::Map::new();
  let mut aliases = vec![];
  for p in &object.props {
    let PropOrSpread::Prop(p) = p else {
      return Err("Palette spread".into());
    };
    match p.as_ref() {
      Prop::KeyValue(p) => {
        colors.insert(name(&p.key)?, literal(&p.value)?);
      }
      Prop::Getter(g) => {
        let alias = name(&g.key)?;
        let target = match alias.as_str() {
          "lightBlue" => "sky",
          "warmGray" => "stone",
          "trueGray" => "neutral",
          "coolGray" => "gray",
          "blueGray" => "slate",
          _ => return Err("Unknown palette getter".into()),
        };
        let Some(Stmt::Return(ret)) = g.body.as_ref().and_then(|b| b.stmts.last()) else {
          return Err("Palette getter lacks alias return".into());
        };
        if !matches!(ret.arg.as_deref(),Some(Expr::Member(m)) if matches!(m.obj.as_ref(),Expr::This(_)) && matches!(&m.prop,MemberProp::Ident(v) if v.sym==target))
        {
          return Err("Palette alias changed".into());
        }
        aliases.push((alias, target));
      }
      _ => return Err("Unsupported palette property".into()),
    }
  }
  // Tailwind v3's effective default theme excludes deprecated public aliases.
  // Validate known getter targets without introducing those extra utility names.
  for (_, target) in aliases {
    if !colors.contains_key(target) {
      return Err("Missing palette alias target".into());
    }
  }
  Ok(Value::Object(colors))
}
fn extract(config: &str, palette_source: &str) -> Result<Value, String> {
  let ast = parse(config)?;
  let mut root = None;
  for item in &ast.body {
    let ModuleItem::Stmt(Stmt::Expr(stmt)) = item else {
      continue;
    };
    let Expr::Assign(assign) = stmt.expr.as_ref() else {
      continue;
    };
    if matches!(&assign.left,AssignTarget::Simple(SimpleAssignTarget::Member(m)) if matches!(m.obj.as_ref(),Expr::Ident(v) if v.sym=="module") && matches!(&m.prop,MemberProp::Ident(v) if v.sym=="exports"))
    {
      if root.is_some() {
        return Err("Multiple module.exports assignments".into());
      }
      root = Some(assign.right.as_ref());
    }
  }
  let Some(Expr::Object(root)) = root else {
    return Err("Missing literal module.exports".into());
  };
  let mut data = serde_json::Map::new();
  let mut variables = false;
  for p in &root.props {
    let PropOrSpread::Prop(p) = p else {
      return Err("Config spread".into());
    };
    let Prop::KeyValue(p) = p.as_ref() else {
      return Err("Dynamic config key".into());
    };
    let key = name(&p.key)?;
    match key.as_str() {
      "corePlugins" | "darkMode" | "theme" => {
        data.insert(key, literal(&p.value)?);
      }
      "content" => {
        literal(&p.value)?;
      }
      "plugins" => {
        let Expr::Array(entries) = p.value.as_ref() else {
          return Err("Dynamic plugins".into());
        };
        for e in &entries.elems {
          let Some(e) = e else {
            return Err("Plugin array hole".into());
          };
          if e.spread.is_some() {
            return Err("Plugin spread".into());
          }
          match e.expr.as_ref() {
            Expr::Ident(v) if v.sym == "addVariablesForColors" => variables = true,
            Expr::Call(c)
              if matches!(&c.callee,Callee::Expr(e) if matches!(e.as_ref(),Expr::Ident(v) if v.sym=="require"))
                && c.args.len() == 1
                && c.args[0].spread.is_none()
                && matches!(c.args[0].expr.as_ref(),Expr::Lit(Lit::Str(v)) if v.value=="tailwindcss-animate") =>
              {}
            _ => return Err("Unreviewed JavaScript plugin".into()),
          }
        }
      }
      _ => return Err(format!("Unreviewed config property {key}")),
    }
  }
  let theme = data
    .get_mut("theme")
    .and_then(Value::as_object_mut)
    .ok_or("Missing literal theme")?;
  if theme.contains_key("colors") {
    return Err("Explicit root colors require a separate migration review".into());
  }
  theme.insert("colors".into(), palette(palette_source)?);
  if data.get("darkMode") == Some(&json!(["class"])) {
    // The Docs host uses data-theme, rather than a Tailwind-specific DOM class.
    data.insert(
      "darkMode".into(),
      json!(["selector", "[data-theme=\"dark\"]"]),
    );
  }
  Ok(json!({"config":data,"colorVariables":variables}))
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
  use sha2::{Digest, Sha256};
  use std::{fs, io::Write};
  let args = std::env::args().skip(1).collect::<Vec<_>>();
  if args.len() != 3 {
    return Err("usage: tailwind_config <old-config.js> <v3-palette.js> <new-options.json>".into());
  }
  let source = fs::read_to_string(&args[0])?;
  let colors = fs::read_to_string(&args[1])?;
  let data = extract(&source, &colors)?;
  let output = serde_json::to_vec_pretty(&data)?;
  let mut file = fs::OpenOptions::new()
    .write(true)
    .create_new(true)
    .open(&args[2])?;
  file.write_all(&output)?;
  file.write_all(b"\n")?;
  println!(
    "{}",
    json!({"status":"STATIC_STYLE_DATA_EXTRACTED","legacyConfigSha256":format!("{:x}",Sha256::digest(source.as_bytes())),"legacyPaletteSha256":format!("{:x}",Sha256::digest(colors.as_bytes())),"colorVariables":data["colorVariables"],"colors":data["config"]["theme"]["colors"].as_object().map(|m|m.len()),"scope":"Rust AST static extraction only; no JS evaluation, CSS compilation or visual acceptance"})
  );
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn extracts_literal_theme_and_v3_palette_without_executing_plugin_functions() {
    let source = "module.exports={corePlugins:{preflight:false},darkMode:['class'],content:['./src/**/*.tsx'],theme:{extend:{colors:{brand:'#fea7df'}},animation:{aurora:'old',aurora:'aurora 60s linear infinite'}},plugins:[addVariablesForColors,require('tailwindcss-animate')]}; function addVariablesForColors(){throw Error('must never execute')}";
    let palette = "const _default={blue:{500:'#3b82f6'},sky:{500:'#0ea5e9'},get lightBlue(){return this.sky;}};";
    let result = extract(source, palette).unwrap();
    assert_eq!(result["colorVariables"], true);
    assert_eq!(
      result["config"]["theme"]["animation"]["aurora"],
      "aurora 60s linear infinite"
    );
    assert_eq!(
      result["config"]["theme"]["colors"]["blue"]["500"],
      "#3b82f6"
    );
    assert!(
      result["config"]["theme"]["colors"]
        .get("lightBlue")
        .is_none()
    );
    assert_eq!(
      result["config"]["darkMode"],
      json!(["selector", "[data-theme=\"dark\"]"])
    );
    assert!(result["config"].get("plugins").is_none());
    assert!(result["config"].get("content").is_none());
  }
  #[test]
  fn rejects_dynamic_theme_and_unreviewed_plugins() {
    let palette = "const _default={blue:{500:'#3b82f6'}};";
    assert!(extract("module.exports={theme:computeTheme()};", palette).is_err());
    assert!(
      extract(
        "module.exports={theme:{},plugins:[require('unknown')]};",
        palette
      )
      .is_err()
    );
    assert!(extract("module.exports={theme:{},other:123};", palette).is_err());
  }
}
