//! Synchronous write cells for source-local exported bindings.
use crate::swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};
use farmfe_core::{
  HashMap,
  config::{FARM_MODULE, FARM_REQUIRE, ModuleFormat, ModuleFormatConfig},
  context::CompilationContext,
  error::Result,
  module::{
    ModuleId,
    meta_data::script::{ModuleExportIdentType, statement::SwcId},
    module_graph::ModuleGraph,
  },
  swc_common::{DUMMY_SP, Mark, SyntaxContext},
  swc_ecma_ast::{
    ArrayPat, AssignPat, AssignTargetPat, CallExpr, Callee, Expr, ForHead, Ident, KeyValuePatProp,
    MemberExpr, MemberProp, Module, ModuleItem, ObjectPat, ObjectPatProp, Pat, PropName,
    SimpleAssignTarget, UpdateExpr,
  },
};

/// Only explicit runtime script entries emit native ESM facades. HTML inputs,
/// internal runtime entries and native library output must not acquire this ABI.
pub fn enabled(graph: &ModuleGraph, context: &CompilationContext) -> bool {
  !context.config.output.target_env.is_library()
    && matches!(
      context.config.output.format,
      ModuleFormatConfig::Single(ModuleFormat::EsModule)
    )
    && graph.entries.keys().any(|id| {
      !farmfe_core::module::is_farm_runtime_module_id(id.relative_path())
        && graph
          .module(id)
          .is_some_and(|module| !module.external && module.module_type.is_script())
    })
}

pub fn origin_key(module: &ModuleId, ident: &SwcId) -> String {
  farmfe_core::serde_json::to_string(&(module.to_string(), ident.sym.to_string())).unwrap()
}

struct WriteCells {
  candidates: HashMap<SwcId, Ident>,
  used: HashMap<SwcId, Ident>,
}
impl WriteCells {
  fn target(&mut self, ident: &Ident) -> Option<MemberExpr> {
    let id = SwcId::from(ident.to_id());
    let cell = self.candidates.get(&id)?.clone();
    self.used.insert(id, cell.clone());
    Some(MemberExpr {
      span: DUMMY_SP,
      obj: Box::new(Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: Callee::Expr(Box::new(Expr::Ident(cell))),
        args: vec![],
        type_args: None,
      })),
      prop: MemberProp::Ident("value".into()),
    })
  }
  fn pattern(&mut self, pat: &mut Pat) {
    match pat {
      Pat::Ident(ident) => {
        if let Some(member) = self.target(&ident.id) {
          *pat = Pat::Expr(Box::new(Expr::Member(member)));
        }
      }
      Pat::Expr(expr) => {
        if let Expr::Ident(ident) = &**expr
          && let Some(member) = self.target(ident)
        {
          **expr = Expr::Member(member);
        }
      }
      Pat::Array(array) => self.array(array),
      Pat::Object(object) => self.object(object),
      Pat::Rest(rest) => self.pattern(&mut rest.arg),
      Pat::Assign(assign) => self.pattern(&mut assign.left),
      _ => {}
    }
  }
  fn array(&mut self, array: &mut ArrayPat) {
    for pat in array.elems.iter_mut().flatten() {
      self.pattern(pat);
    }
  }
  fn object(&mut self, object: &mut ObjectPat) {
    for prop in &mut object.props {
      match prop {
        ObjectPatProp::KeyValue(pair) => self.pattern(&mut pair.value),
        ObjectPatProp::Rest(rest) => self.pattern(&mut rest.arg),
        ObjectPatProp::Assign(assign) => {
          if let Some(member) = self.target(&assign.key.id) {
            let mut pat = Pat::Expr(Box::new(Expr::Member(member)));
            if let Some(value) = assign.value.take() {
              pat = Pat::Assign(AssignPat {
                span: assign.span,
                left: Box::new(pat),
                right: value,
              });
            }
            *prop = ObjectPatProp::KeyValue(KeyValuePatProp {
              key: PropName::Ident(assign.key.id.sym.clone().into()),
              value: Box::new(pat),
            });
          }
        }
      }
    }
  }
}
impl VisitMut for WriteCells {
  fn visit_mut_assign_target_pat(&mut self, target: &mut AssignTargetPat) {
    target.visit_mut_children_with(self);
    match target {
      AssignTargetPat::Array(array) => self.array(array),
      AssignTargetPat::Object(object) => self.object(object),
      _ => {}
    }
  }
  fn visit_mut_for_head(&mut self, head: &mut ForHead) {
    head.visit_mut_children_with(self);
    if let ForHead::Pat(pat) = head {
      self.pattern(pat);
    }
  }
  fn visit_mut_simple_assign_target(&mut self, target: &mut SimpleAssignTarget) {
    target.visit_mut_children_with(self);
    if let SimpleAssignTarget::Ident(ident) = target
      && let Some(member) = self.target(&ident.id)
    {
      *target = SimpleAssignTarget::Member(member);
    }
  }
  fn visit_mut_update_expr(&mut self, update: &mut UpdateExpr) {
    update.visit_mut_children_with(self);
    if let Expr::Ident(ident) = &*update.arg
      && let Some(member) = self.target(ident)
    {
      *update.arg = Expr::Member(member);
    }
  }
}
struct GeneratedNames {
  value: Ident,
  cell: Ident,
  cache: Ident,
  private: SyntaxContext,
  unresolved: SyntaxContext,
}
impl VisitMut for GeneratedNames {
  fn visit_mut_ident(&mut self, ident: &mut Ident) {
    match ident.sym.as_ref() {
      "__FARM_VALUE__" => *ident = self.value.clone(),
      "__FARM_CELL__" => *ident = self.cell.clone(),
      "__FARM_CACHE__" => *ident = self.cache.clone(),
      "__FARM_NEXT__" => ident.ctxt = self.private,
      "__FARM_REQUIRE__" => {
        ident.sym = FARM_REQUIRE.into();
        ident.ctxt = self.unresolved;
      }
      "__FARM_MODULE__" => {
        ident.sym = FARM_MODULE.into();
        ident.ctxt = self.unresolved;
      }
      _ => {}
    }
  }
}

struct ImmutableBindings(farmfe_core::HashSet<SwcId>);
impl Visit for ImmutableBindings {
  fn visit_var_decl(&mut self, declaration: &farmfe_core::swc_ecma_ast::VarDecl) {
    if declaration.kind == farmfe_core::swc_ecma_ast::VarDeclKind::Const {
      for declarator in &declaration.decls {
        let mut collector = super::idents_collector::DefinedIdentsCollector::new();
        declarator.name.visit_with(&mut collector);
        self.0.extend(collector.defined_idents);
      }
    }
    declaration.visit_children_with(self);
  }
  fn visit_import_decl(&mut self, import: &farmfe_core::swc_ecma_ast::ImportDecl) {
    for specifier in &import.specifiers {
      self.0.insert(specifier.local().to_id().into());
    }
  }
}

/// Source-local plans are recorded before resource-cache lookup as well as
/// rendering, so a cache hit cannot lose HMR's previously mutable bindings.
pub fn remember_origins(
  module_id: &ModuleId,
  graph: &ModuleGraph,
  context: &CompilationContext,
) -> Vec<String> {
  if !enabled(graph, context) {
    return vec![];
  }
  type Plans = HashMap<ModuleId, farmfe_core::HashSet<String>>;
  type Scans = HashMap<ModuleId, (String, Vec<String>)>;
  let Some(source) = graph.module(module_id) else {
    return vec![];
  };
  if source.external || !source.module_type.is_script() {
    return vec![];
  }
  let meta = source.meta.as_script();
  let mut contract: Vec<_> = meta
    .get_export_idents()
    .into_iter()
    .map(|(name, export)| {
      let origin = export.as_internal();
      (
        name,
        origin.module_id.to_string(),
        origin.ident.sym.to_string(),
        format!("{:?}", origin.export_type),
      )
    })
    .collect();
  contract.sort();
  let scan_key = format!("{}:{:?}", source.content_hash, contract);
  let mut custom = context.custom.lock();
  let scans = custom
    .entry("farm.entry-live-scans.v1".into())
    .or_insert_with(|| Box::new(Scans::default()))
    .downcast_mut::<Scans>()
    .expect("private live-origin scan type");
  if let Some((previous, names)) = scans.get(module_id)
    && previous == &scan_key
  {
    return names.clone();
  }
  let mut immutable = ImmutableBindings(Default::default());
  meta.ast.visit_with(&mut immutable);
  let mut cells = WriteCells {
    candidates: HashMap::default(),
    used: HashMap::default(),
  };
  for (_, export) in meta.get_export_idents() {
    let origin = export.as_internal();
    if origin.module_id == *module_id
      && origin.export_type == ModuleExportIdentType::Declaration
      && !immutable.0.contains(&origin.ident)
    {
      cells
        .candidates
        .insert(origin.ident.clone(), origin.ident.clone().into());
    }
  }
  meta.ast.clone().visit_mut_with(&mut cells);
  let plans = custom
    .entry("farm.entry-live-origins.v1".into())
    .or_insert_with(|| Box::new(Plans::default()))
    .downcast_mut::<Plans>()
    .expect("private live-origin plan type");
  let names = plans.entry(module_id.clone()).or_default();
  names.extend(cells.used.keys().map(|id| id.sym.to_string()));
  let mut sorted: Vec<_> = names.iter().cloned().collect();
  sorted.sort();
  custom
    .get_mut("farm.entry-live-scans.v1")
    .expect("scan map initialized")
    .downcast_mut::<Scans>()
    .expect("private live-origin scan type")
    .insert(module_id.clone(), (scan_key, sorted.clone()));
  sorted
}

pub fn instrument(
  module_id: &ModuleId,
  ast: &mut Module,
  graph: &ModuleGraph,
  context: &CompilationContext,
) -> Result<Vec<SwcId>> {
  if !enabled(graph, context) {
    return Ok(vec![]);
  }
  let tracked = remember_origins(module_id, graph, context);
  let source = graph.module(module_id).expect("rendered module must exist");
  let meta = source.meta.as_script();
  let private = SyntaxContext::empty().apply_mark(Mark::new());
  let unresolved = SyntaxContext::empty().apply_mark(Mark::from_u32(meta.unresolved_mark));
  let mut cells = WriteCells {
    candidates: HashMap::default(),
    used: HashMap::default(),
  };
  let mut immutable = ImmutableBindings(Default::default());
  meta.ast.visit_with(&mut immutable);
  for (_, exported) in meta.get_export_idents() {
    let origin = exported.as_internal();
    if origin.module_id == *module_id
      && origin.export_type == ModuleExportIdentType::Declaration
      && !immutable.0.contains(&origin.ident)
    {
      let cell = Ident::new(
        format!("__farm_live_{}", origin.ident.sym).into(),
        DUMMY_SP,
        private,
      );
      cells.candidates.insert(origin.ident.clone(), cell);
    }
  }
  ast.visit_mut_with(&mut cells);
  let mut used: Vec<_> = cells.used.into_iter().collect();
  used.sort_by(|a, b| a.0.sym.cmp(&b.0.sym));
  let mut prefix = Vec::new();
  let mut generated = Vec::new();
  let mut suffix = Vec::new();
  let mut keys = Vec::new();
  for (value, cell) in used {
    let key = farmfe_core::serde_json::to_string(&origin_key(module_id, &value)).unwrap();
    let cache = Ident::new(format!("{}_cache", cell.sym).into(), DUMMY_SP, private);
    generated.push(SwcId::from(cell.clone()));
    generated.push(SwcId::from(cache.clone()));
    keys.push(origin_key(module_id, &value));
    let mut stmt = super::parse_stmt(
      "farm-entry-write-cell",
      &format!(
        "function __FARM_CELL__(){{return __FARM_CACHE__||(__FARM_CACHE__=__FARM_REQUIRE__.entryBindings.bind({key},__FARM_MODULE__,function(){{return __FARM_VALUE__;}},function(__FARM_NEXT__){{__FARM_VALUE__=__FARM_NEXT__;}}));}}"
      ),
    )?;
    stmt.visit_mut_with(&mut GeneratedNames {
      value: value.into(),
      cell,
      cache: cache.clone(),
      private,
      unresolved,
    });
    prefix.push(ModuleItem::Stmt(stmt));
    let mut cache_decl = super::parse_stmt("farm-entry-cell-cache", "var __FARM_CACHE__;")?;
    cache_decl.visit_mut_with(&mut GeneratedNames {
      value: cache.clone(),
      cell: cache.clone(),
      cache: cache.clone(),
      private,
      unresolved,
    });
    prefix.push(ModuleItem::Stmt(cache_decl));
    let mut initialize = super::parse_stmt("farm-entry-cell-initialize", "__FARM_CELL__();")?;
    initialize.visit_mut_with(&mut GeneratedNames {
      value: cache.clone(),
      cell: generated[generated.len() - 2].clone().into(),
      cache: cache.clone(),
      private,
      unresolved,
    });
    suffix.push(ModuleItem::Stmt(initialize));
  }
  // If HMR replaces a previously mutable binding with a constant/no-write
  // binding, publish the new initialized value without inventing a setter.
  let mut seeded = farmfe_core::HashSet::default();
  for (_, exported) in meta.get_export_idents() {
    let origin = exported.as_internal();
    if origin.module_id != *module_id
      || origin.export_type != ModuleExportIdentType::Declaration
      || !tracked.contains(&origin.ident.sym.to_string())
    {
      continue;
    }
    let key_value = origin_key(module_id, &origin.ident);
    if keys.contains(&key_value) || !seeded.insert(key_value.clone()) {
      continue;
    }
    let key = farmfe_core::serde_json::to_string(&key_value).unwrap();
    let mut seed = super::parse_stmt(
      "farm-entry-replacement-seed",
      &format!(
        "__FARM_REQUIRE__.entryBindings.seed({key},__FARM_MODULE__,function(){{return __FARM_VALUE__;}});"
      ),
    )?;
    seed.visit_mut_with(&mut GeneratedNames {
      value: origin.ident.clone().into(),
      cell: origin.ident.clone().into(),
      cache: origin.ident.clone().into(),
      private,
      unresolved,
    });
    suffix.push(ModuleItem::Stmt(seed));
  }
  if !keys.is_empty() {
    let mut complete = super::parse_stmt(
      "farm-entry-cell-complete",
      &format!(
        "__FARM_REQUIRE__.entryBindings.complete(__FARM_MODULE__,{});",
        farmfe_core::serde_json::to_string(&keys).unwrap()
      ),
    )?;
    complete.visit_mut_with(&mut GeneratedNames {
      value: Ident::new("unused".into(), DUMMY_SP, private),
      cell: Ident::new("unused".into(), DUMMY_SP, private),
      cache: Ident::new("unused".into(), DUMMY_SP, private),
      private,
      unresolved,
    });
    suffix.push(ModuleItem::Stmt(complete));
  }
  prefix.append(&mut ast.body);
  prefix.append(&mut suffix);
  ast.body = prefix;
  Ok(generated)
}

/// Follow explicit static imports/reexports to their source-local binding.
pub fn resolve_origin(
  graph: &ModuleGraph,
  module_id: &ModuleId,
  name: &str,
) -> Option<(ModuleId, SwcId)> {
  fn resolve(
    graph: &ModuleGraph,
    module_id: &ModuleId,
    name: &str,
    seen: &mut farmfe_core::HashSet<(ModuleId, String)>,
  ) -> Option<(ModuleId, SwcId)> {
    use farmfe_core::{
      module::meta_data::script::ModuleReExportIdentType,
      swc_ecma_ast::{ImportSpecifier, ModuleDecl, ModuleExportName},
    };
    if !seen.insert((module_id.clone(), name.to_string())) {
      return None;
    }
    let module = graph.module(module_id)?;
    if module.external || !module.module_type.is_script() {
      return None;
    }
    let meta = module.meta.as_script();
    if let Some(reexport) = meta.reexport_ident_map.get(name) {
      return match reexport {
        ModuleReExportIdentType::FromExportAll(from) => resolve(graph, from, name, seen),
        ModuleReExportIdentType::FromExportNamed {
          local,
          from_module_id,
        } => resolve(graph, from_module_id, local, seen),
      };
    }
    let exported = meta.export_ident_map.get(name)?.as_internal();
    let owner = graph.module(&exported.module_id)?;
    if owner.external || !owner.module_type.is_script() {
      return None;
    }
    for item in &owner.meta.as_script().ast.body {
      if let ModuleItem::ModuleDecl(ModuleDecl::Import(import)) = item {
        for specifier in &import.specifiers {
          let (local, imported) = match specifier {
            ImportSpecifier::Named(named) => (
              &named.local,
              match &named.imported {
                Some(ModuleExportName::Ident(id)) => id.sym.to_string(),
                Some(ModuleExportName::Str(value)) => value.value.to_string_lossy().into_owned(),
                None => named.local.sym.to_string(),
              },
            ),
            ImportSpecifier::Default(default) => (&default.local, "default".to_string()),
            ImportSpecifier::Namespace(_) => continue,
          };
          if SwcId::from(local.to_id()) == exported.ident {
            let dependency = graph.get_dep_by_source_optional(
              &owner.id,
              &import.src.value.to_string_lossy(),
              None,
            )?;
            return resolve(graph, &dependency, &imported, seen);
          }
        }
      }
    }
    if exported.export_type != ModuleExportIdentType::Declaration {
      return None;
    }
    Some((exported.module_id.clone(), exported.ident.clone()))
  }
  resolve(graph, module_id, name, &mut Default::default())
}
