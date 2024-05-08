use std::{
  cell::RefCell,
  collections::{HashMap, HashSet},
  hash::Hash,
  rc::Rc,
  sync::Arc,
};

use farmfe_core::{
  context::CompilationContext,
  enhanced_magic_string::bundle::Bundle,
  error::{CompilationError, Result},
  module::{module_graph::ModuleGraph, ModuleId, ModuleType},
  resource::resource_pot::{ResourcePot, ResourcePotId, ResourcePotType},
  swc_ecma_ast::Id,
};

pub use crate::resource_pot_to_bundle::bundle_analyzer::BundleAnalyzer;

use self::{
  bundle::ModuleAnalyzerManager, modules_analyzer::module_analyzer::ModuleAnalyzer,
  uniq_name::BundleVariable,
};

mod bundle;
mod common;
mod defined_idents_collector;
mod modules_analyzer;
mod targets;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Var {
  var: Id,
  rename: Option<String>,
  removed: bool,
}

impl Var {
  pub fn new(id: Id) -> Self {
    Var {
      var: id,
      ..Default::default()
    }
  }

  pub fn render_name(&self) -> String {
    if let Some(rename) = self.rename.as_ref() {
      rename.clone()
    } else {
      self.var.0.to_string()
    }
  }

  pub fn origin_name(&self) -> String {
    self.var.0.to_string()
  }
}

impl Hash for Var {
  fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
    self.var.hash(state);
  }
}

mod bundle_analyzer;
mod bundle_external;

mod uniq_name;

pub struct SharedBundle<'a> {
  pub bundle_map: HashMap<ResourcePotId, BundleAnalyzer<'a>>,
  module_analyzer_manager: ModuleAnalyzerManager,
  module_graph: &'a ModuleGraph,
  context: &'a Arc<CompilationContext>,
  bundle_variables: Rc<RefCell<BundleVariable>>,
  order_index_map: HashMap<ModuleId, usize>,
}

///
/// TODO:
/// 1. multiple bundle
/// 2. commonjs
/// 3. dynamic bundle
/// 4. multiple environment process
///
impl<'a> SharedBundle<'a> {
  pub fn new(
    resource_pots: Vec<&'a ResourcePot>,
    module_graph: &'a ModuleGraph,
    context: &'a Arc<CompilationContext>,
  ) -> Self {
    let mut module_analyzer_map: HashMap<ModuleId, ModuleAnalyzer> = HashMap::new();
    let mut bundle_map: HashMap<ResourcePotId, BundleAnalyzer> = HashMap::new();

    let bundle_variables = Rc::new(RefCell::new(BundleVariable::new()));

    let (toposort_modules, _) = module_graph.toposort();

    let mut order_map = HashMap::new();

    for (index, module_id) in toposort_modules.into_iter().enumerate() {
      order_map.insert(module_id, index);
    }

    // 1. analyze resource pot
    for resource_pot in resource_pots.iter() {
      if !(matches!(
        resource_pot.resource_pot_type,
        ResourcePotType::Js | ResourcePotType::Runtime
      )) {
        continue;
      }

      // 1-1. analyze bundle
      let mut bundle_analyzer = BundleAnalyzer::new(
        &resource_pot,
        &module_graph,
        &context,
        bundle_variables.clone(),
      );

      bundle_variables
        .borrow_mut()
        .with_namespace(resource_pot.id.clone(), |_| {
          for module_id in resource_pot.modules() {
            let is_dynamic = module_graph.is_dynamic(module_id);
            let is_entry = resource_pot
              .entry_module
              .as_ref()
              .is_some_and(|item| item == module_id);
            let module = module_graph.module(module_id).unwrap();
            let is_runtime = matches!(module.module_type, ModuleType::Runtime);

            module_analyzer_map.insert(
              module_id.clone(),
              // 1-2. analyze bundle module
              ModuleAnalyzer::new(
                module,
                &context,
                resource_pot.id.clone(),
                is_entry,
                is_dynamic,
                is_runtime,
              )
              .unwrap(),
            );
          }

          // 1-3. order bundle module
          bundle_analyzer.build_module_order(&order_map);

          bundle_map.insert(resource_pot.id.clone(), bundle_analyzer);
        });
    }

    // modules manager
    let module_analyzer_manager = ModuleAnalyzerManager::new(module_analyzer_map);

    Self {
      module_analyzer_manager,
      bundle_map,
      module_graph,
      context,
      bundle_variables,
      order_index_map: order_map,
    }
  }

  // 2-1 extract module data from ast
  fn extract_modules(&mut self) -> Result<()> {
    for resource_pot_id in self
      .module_analyzer_manager
      .module_map
      .values()
      .map(|item| item.resource_pot_id.clone())
      .collect::<HashSet<_>>()
    {
      let bundle = self
        .bundle_map
        .get_mut(&resource_pot_id)
        .map(Ok)
        .unwrap_or_else(|| {
          Err(CompilationError::GenericError(format!(
            "fetch unknown resource pot {:?} failed",
            resource_pot_id
          )))
        })?;

      bundle
        .bundle_variable
        .borrow_mut()
        .set_namespace(resource_pot_id);

      self.module_analyzer_manager.extract_modules_statements(
        &bundle.ordered_modules,
        &self.context,
        &self.module_graph,
        bundle.bundle_variable.borrow_mut(),
      )?;
    }

    Ok(())
  }

  // 2-2 process common module data
  fn link_modules(&mut self) -> Result<()> {
    self.module_analyzer_manager.link(
      &mut self.bundle_variables.borrow_mut(),
      &self.module_graph,
      &self.context,
    );

    Ok(())
  }

  // 2-3 start process bundle
  fn render_bundle(&mut self) -> Result<()> {
    for bundle_analyzer in self.bundle_map.values_mut() {
      // println!("// bundle_analyzer: {:?}", bundle_analyzer.resource_pot.id);
      bundle_analyzer
        .bundle_variable
        .borrow_mut()
        .set_namespace(bundle_analyzer.resource_pot.id.clone());

      bundle_analyzer.render(&mut self.module_analyzer_manager, &self.order_index_map)?;
    }

    Ok(())
  }

  // 2. start process bundle
  pub fn render(&mut self) -> Result<()> {
    self.extract_modules()?;

    self.link_modules()?;

    self.render_bundle()?;

    Ok(())
  }

  pub fn codegen(&mut self, resource_pot_id: &String) -> Result<Bundle> {
    let bundle = self.bundle_map.get_mut(resource_pot_id).unwrap();

    let bundle = bundle.codegen(&mut self.module_analyzer_manager)?;

    Ok(bundle)
  }
}
