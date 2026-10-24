use farmfe_core::{
  error::CompilationError,
  swc_common::{DUMMY_SP, FileName, Globals, source_map::SourceMapGenConfig},
  swc_ecma_ast::{
    BlockStmtOrExpr, Bool, CallExpr, Callee, CondExpr, EsVersion, Expr, ExprOrSpread, Ident,
    ImportNamedSpecifier, ImportSpecifier, KeyValueProp, Lit, Module, ModuleDecl, ModuleExportName,
    ModuleItem, ParenExpr, PropName, SeqExpr,
  },
  swc_ecma_parser::{Syntax, TsSyntax},
};
use farmfe_toolkit::{
  script::{codegen_module, parse_module, swc_try_with::resolve_module_mark},
  sourcemap::{SourceMap, SourceMapBuilder},
  swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith},
};
use std::{collections::HashMap, sync::Arc};

fn helpers(module: &Module) -> HashMap<String, Ident> {
  let mut result = HashMap::new();
  for item in &module.body {
    if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
      if import.src.value.as_str() != Some("vue") {
        continue;
      }
      for specifier in &import.specifiers {
        if let ImportSpecifier::Named(named) = specifier {
          let name = match &named.imported {
            Some(ModuleExportName::Ident(id)) => id.sym.to_string(),
            Some(ModuleExportName::Str(name)) => name.value.to_string_lossy().into_owned(),
            None => named.local.sym.to_string(),
          };
          result.insert(name, named.local.clone());
        }
      }
    }
  }
  result
}

#[derive(Default)]
struct Names(std::collections::HashSet<String>);
impl Visit for Names {
  fn visit_ident(&mut self, id: &Ident) {
    self.0.insert(id.sym.to_string());
  }
}

struct ListBlocks {
  helpers: HashMap<String, Ident>,
  changed: bool,
}
impl ListBlocks {
  fn is_helper(&self, callee: &Callee, name: &str) -> bool {
    matches!(callee, Callee::Expr(expr) if matches!(&**expr, Expr::Ident(id) if self.helpers.get(name).is_some_and(|helper| helper.to_id() == id.to_id())))
  }
  fn is_block(&self, expr: &Expr) -> bool {
    match expr {
      Expr::Paren(paren) => self.is_block(&paren.expr),
      Expr::Seq(seq) => seq.exprs.last().is_some_and(|expr| self.is_block(expr)),
      Expr::Call(call) => {
        self.is_helper(&call.callee, "createElementBlock")
          || self.is_helper(&call.callee, "createBlock")
      }
      _ => false,
    }
  }
  fn repair_comment(&mut self, expr: &mut Expr) {
    if let Expr::Paren(paren) = expr {
      self.repair_comment(&mut paren.expr);
      return;
    }
    let Expr::Call(call) = expr else {
      return;
    };
    if self.is_helper(&call.callee, "createCommentVNode") && call.args.len() == 1 {
      // A conditional block needs a matching placeholder in its parent's
      // dynamicChildren when the element branch is absent.
      call.args.push(ExprOrSpread {
        spread: None,
        expr: Box::new(Expr::Lit(Lit::Bool(Bool {
          span: DUMMY_SP,
          value: true,
        }))),
      });
      self.changed = true;
    }
  }
  fn repair_item(&mut self, expr: &mut Box<Expr>) {
    if let Expr::Paren(paren) = &mut **expr {
      self.repair_item(&mut paren.expr);
      return;
    }
    let Expr::Call(call) = &mut **expr else {
      return;
    };
    if self.is_helper(&call.callee, "withDirectives") {
      if let Some(item) = call.args.first_mut() {
        self.repair_item(&mut item.expr);
      }
      return;
    }
    let block = if self.is_helper(&call.callee, "createElementVNode") {
      self.helpers.get("createElementBlock")
    } else if self.is_helper(&call.callee, "createVNode") {
      self.helpers.get("createBlock")
    } else {
      None
    };
    let (Some(block), Some(open)) = (block, self.helpers.get("openBlock")) else {
      return;
    };
    call.callee = Callee::Expr(Box::new(Expr::Ident(block.clone())));
    let span = call.span;
    let node = expr.clone();
    **expr = Expr::Paren(ParenExpr {
      span,
      expr: Box::new(Expr::Seq(SeqExpr {
        span,
        exprs: vec![
          Box::new(Expr::Call(CallExpr {
            span,
            ctxt: Default::default(),
            callee: Callee::Expr(Box::new(Expr::Ident(open.clone()))),
            args: vec![],
            type_args: None,
          })),
          node,
        ],
      })),
    });
    self.changed = true;
  }
}
impl VisitMut for ListBlocks {
  fn visit_mut_cond_expr(&mut self, conditional: &mut CondExpr) {
    conditional.visit_mut_children_with(self);
    if self.is_block(&conditional.cons) {
      self.repair_comment(&mut conditional.alt);
    }
    if self.is_block(&conditional.alt) {
      self.repair_comment(&mut conditional.cons);
    }
  }
  fn visit_mut_call_expr(&mut self, call: &mut CallExpr) {
    call.visit_mut_children_with(self);
    if !self.is_helper(&call.callee, "renderList") {
      return;
    }
    if let Some(callback) = call.args.get_mut(1)
      && let Expr::Arrow(arrow) = &mut *callback.expr
      && let BlockStmtOrExpr::Expr(body) = &mut *arrow.body
    {
      self.repair_item(body);
    }
  }
}

struct ColumnMaps;
impl SourceMapGenConfig for ColumnMaps {
  fn file_name_to_source(&self, filename: &FileName) -> String {
    filename.to_string()
  }
  fn inline_sources_content(&self, _: &FileName) -> bool {
    true
  }
  fn emit_columns(&self, _: &FileName) -> bool {
    true
  }
}

struct ScopeIds {
  scope: String,
  changed: bool,
}
impl VisitMut for ScopeIds {
  fn visit_mut_key_value_prop(&mut self, prop: &mut KeyValueProp) {
    prop.visit_mut_children_with(self);
    let is_scope = match &prop.key {
      PropName::Ident(id) => id.sym == "__scopeId",
      PropName::Str(value) => value.value.as_str() == Some("__scopeId"),
      _ => false,
    };
    if is_scope
      && let Expr::Lit(farmfe_core::swc_ecma_ast::Lit::Str(value)) = &mut *prop.value
      && value.value.as_str() != Some(&self.scope)
    {
      value.value = self.scope.clone().into();
      value.raw = None;
      self.changed = true;
    }
  }
}

#[cfg(test)]
pub fn repair_list_blocks(
  code: String,
  source_map: Option<String>,
  filename: &str,
) -> Result<(String, Option<String>), CompilationError> {
  normalize_vue_code(code, source_map, filename, None)
}

pub fn normalize_vue_code(
  code: String,
  source_map: Option<String>,
  filename: &str,
  stable_scope: Option<&str>,
) -> Result<(String, Option<String>), CompilationError> {
  if !code.contains("renderList") && !code.contains("createCommentVNode") && stable_scope.is_none()
  {
    return Ok((code, source_map));
  }
  let mut parsed = parse_module(
    &filename.into(),
    Arc::new(code.clone()),
    Syntax::Typescript(TsSyntax::default()),
    EsVersion::EsNext,
  )?;
  let imports = helpers(&parsed.ast);
  if !imports.contains_key("renderList")
    && !imports.contains_key("createCommentVNode")
    && stable_scope.is_none()
  {
    return Ok((code, source_map));
  }
  // Components use createBlock, while native elements use createElementBlock.
  // Pick a name that cannot collide with any user binding in the SFC.
  if imports.contains_key("renderList")
    && imports.contains_key("createVNode")
    && !imports.contains_key("createBlock")
  {
    let mut names = Names::default();
    parsed.ast.visit_with(&mut names);
    let mut local = "_farm_vue_listBlock".to_string();
    while names.0.contains(&local) {
      local.push('_');
    }
    for item in &mut parsed.ast.body {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item
        && import.src.value.as_str() == Some("vue")
        && !import.type_only
      {
        import
          .specifiers
          .push(ImportSpecifier::Named(ImportNamedSpecifier {
            span: DUMMY_SP,
            local: Ident::new_no_ctxt(local.into(), DUMMY_SP),
            imported: Some(ModuleExportName::Ident(Ident::new_no_ctxt(
              "createBlock".into(),
              DUMMY_SP,
            ))),
            is_type_only: false,
          }));
        break;
      }
    }
  }
  resolve_module_mark(&mut parsed.ast, true, &Globals::new());
  let mut visitor = ListBlocks {
    helpers: helpers(&parsed.ast),
    changed: false,
  };
  parsed.ast.visit_mut_with(&mut visitor);
  let mut scope_changed = false;
  if let Some(scope) = stable_scope {
    let mut scope_ids = ScopeIds {
      scope: scope.to_string(),
      changed: false,
    };
    parsed.ast.visit_mut_with(&mut scope_ids);
    scope_changed = scope_ids.changed;
  }
  if !visitor.changed && !scope_changed {
    return Ok((code, source_map));
  }
  let mut mappings = vec![];
  let bytes = codegen_module(
    &parsed.ast,
    parsed.source_map.clone(),
    Some(&mut mappings),
    Default::default(),
    None,
  )
  .map_err(|error| CompilationError::GenericError(format!("Generate Vue list blocks: {error}")))?;
  let code =
    String::from_utf8(bytes).map_err(|error| CompilationError::GenericError(error.to_string()))?;
  let source_map = source_map
    .map(|original| {
      let original = SourceMap::from_slice(original.as_bytes())
        .map_err(|error| CompilationError::GenericError(format!("Read Vue source map: {error}")))?;
      let generated = parsed
        .source_map
        .build_source_map(&mappings, None, ColumnMaps);
      let mut bytes = vec![];
      generated
        .to_writer(&mut bytes)
        .map_err(|error| CompilationError::GenericError(error.to_string()))?;
      let generated = SourceMap::from_slice(&bytes)
        .map_err(|error| CompilationError::GenericError(error.to_string()))?;
      let mut builder = SourceMapBuilder::new(original.get_file().cloned());
      for token in generated.tokens() {
        if let Some(source) = original.lookup_token(token.get_src_line(), token.get_src_col()) {
          if source.get_source().is_none() {
            continue;
          }
          let mapped = builder.add(
            token.get_dst_line(),
            token.get_dst_col(),
            source.get_src_line(),
            source.get_src_col(),
            source.get_source().cloned(),
            source.get_name().cloned(),
            source.is_range(),
          );
          builder.set_source_contents(
            mapped.src_id,
            original.get_source_contents(source.get_src_id()).cloned(),
          );
        }
      }
      let mut bytes = vec![];
      builder
        .into_sourcemap()
        .to_writer(&mut bytes)
        .map_err(|error| CompilationError::GenericError(error.to_string()))?;
      String::from_utf8(bytes).map_err(|error| CompilationError::GenericError(error.to_string()))
    })
    .transpose()?;
  Ok((code, source_map))
}

#[cfg(test)]
mod tests {
  // cspell:ignore AACA
  use super::*;

  #[test]
  fn conditional_comment_is_a_block_without_a_list() {
    let code = r#"import { createElementBlock as block, openBlock as open, createCommentVNode as comment } from 'vue';
export const render = () => ready ? (open(), block('p', null, 'empty')) : comment('v-if');"#;
    let (output, _) = repair_list_blocks(code.into(), None, "Conditional.vue").unwrap();
    let compact: String = output.chars().filter(|c| !c.is_whitespace()).collect();
    assert!(
      compact.contains("comment('v-if',true)"),
      "conditional branches must both register a block: {output}"
    );
  }

  #[test]
  fn conditional_comment_does_not_rewrite_shadowed_or_ordinary_calls() {
    let code = r#"import { createElementBlock as block, openBlock as open, createCommentVNode as comment } from 'vue';
export const render = () => [comment('ordinary'), ready ? 'text' : comment('text-branch')];
export function custom(comment) { return ready ? (open(), block('p')) : comment('v-if'); }"#;
    let (output, _) = repair_list_blocks(code.into(), None, "Conditional.vue").unwrap();
    assert_eq!(output, code);
  }

  #[test]
  fn native_list_items_have_independent_update_blocks() {
    let code = r#"import { renderList as _renderList, createElementVNode as _createElementVNode, createElementBlock as _createElementBlock, openBlock as _openBlock } from 'vue';
export const render = () => _renderList(rows, row => _createElementVNode('tr', { key: row.id }, [_createElementVNode('td', null, row.stock, 1)]));"#;
    let (output, _) = repair_list_blocks(code.to_string(), None, "List.vue").unwrap();
    assert!(
      output.contains("_createElementBlock('tr'"),
      "list rows need their own dynamic-children block: {output}"
    );
    assert!(output.contains("_openBlock()"));
    assert!(
      output.contains("_createElementVNode('td'"),
      "ordinary children must retain vnode generation"
    );
  }

  #[test]
  fn shadowed_helpers_are_not_rewritten() {
    let code = r#"import { renderList as list, createElementVNode as vnode, createElementBlock as block, openBlock as open } from 'vue';
export function custom(list, vnode) { return list(rows, row => vnode('tr', {key: row.id})); }"#;
    let (output, _) = repair_list_blocks(code.to_string(), None, "List.vue").unwrap();
    assert_eq!(output, code);
  }

  #[test]
  fn rewritten_maps_still_reference_original_sfc() {
    let code = r#"import { renderList as list, createElementVNode as vnode, createElementBlock as block, openBlock as open } from 'vue';
export const render = () => list(rows, row => vnode('tr', {key: row.id}));"#;
    let original_map = r#"{"version":3,"sources":["Original.vue"],"sourcesContent":["<template>original</template>"],"names":[],"mappings":"AAAA;AACA"}"#;
    let (output, map) =
      repair_list_blocks(code.to_string(), Some(original_map.to_string()), "List.vue").unwrap();
    assert!(output.contains("block('tr'"));
    let map: farmfe_core::serde_json::Value =
      farmfe_core::serde_json::from_str(&map.unwrap()).unwrap();
    assert_eq!(map["sources"][0], "Original.vue");
    assert_eq!(map["sourcesContent"][0], "<template>original</template>");
    assert!(!map["mappings"].as_str().unwrap().is_empty());
  }

  #[test]
  fn directive_wrapped_list_items_receive_update_blocks() {
    let code = r#"import { renderList as list, createElementVNode as vnode, createElementBlock as block, openBlock as open, withDirectives as directives } from 'vue';
export const render = () => list(rows, row => directives(vnode('tr', {key: row.id}), []));"#;
    let (output, _) = repair_list_blocks(code.to_string(), None, "List.vue").unwrap();
    assert!(output.contains("block('tr'"));
  }

  #[test]
  fn unmapped_generated_helpers_do_not_panic() {
    let code = r#"import { renderList as list, createElementVNode as vnode, createElementBlock as block, openBlock as open } from 'vue';
export const render = () => list(rows, row => vnode('tr', {key: row.id}));"#;
    let original_map = r#"{"version":3,"sources":["Original.vue"],"sourcesContent":["<template>original</template>"],"names":[],"mappings":"A;AACA"}"#;
    assert!(
      repair_list_blocks(code.to_string(), Some(original_map.to_string()), "List.vue").is_ok()
    );
  }

  #[test]
  fn component_list_items_use_component_blocks() {
    let code = r#"import { renderList as list, createVNode as vnode, createElementBlock as block, openBlock as open } from 'vue';
export const render = () => list(rows, row => vnode(Panel, {key: row.id, stock: row.stock}));"#;
    let (output, _) = repair_list_blocks(code.to_string(), None, "List.vue").unwrap();
    assert!(output.contains("createBlock as _farm_vue_listBlock"));
    assert!(output.contains("_farm_vue_listBlock(Panel"));
  }
}
