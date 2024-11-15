use std::{
  cell::RefCell,
  collections::HashMap,
  hash::Hash,
  rc::Rc,
  sync::{Arc, Mutex},
};

use farmfe_core::{
  context::CompilationContext,
  enhanced_magic_string::bundle::Bundle,
  error::{CompilationError, MapCompletionError, Result},
  farm_profile_function, farm_profile_scope,
  module::{module_graph::ModuleGraph, ModuleId, ModuleType},
  rayon::iter::{IntoParallelIterator, ParallelIterator},
  resource::resource_pot::{ResourcePot, ResourcePotId, ResourcePotType},
  swc_ecma_ast::Id,
};
pub use polyfill::{Polyfill, SimplePolyfill};

pub use crate::resource_pot_to_bundle::bundle::bundle_analyzer::BundleAnalyzer;

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
  // maybe global variable, function params name, only as the slot for UniqName
  placeholder: bool,
}

impl Var {
  pub fn new(id: Id) -> Self {
    Var {
      var: id,
      ..Default::default()
    }
  }

  pub fn render_name(&self) -> String {
    if !self.placeholder
      && let Some(rename) = self.rename.as_ref()
    {
      rename.clone()
    } else {
      self.origin_name()
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

mod polyfill;
mod uniq_name;

pub type BundleMap<'a> = HashMap<ResourcePotId, BundleAnalyzer<'a>>;

pub struct SharedBundle<'a> {
  pub bundle_map: BundleMap<'a>,
  module_analyzer_manager: ModuleAnalyzerManager<'a>,
  module_graph: &'a ModuleGraph,
  context: &'a Arc<CompilationContext>,
  pub bundle_variables: Rc<RefCell<BundleVariable>>,
  order_index_map: HashMap<ModuleId, usize>,
  order_resource_pot: Vec<ResourcePotId>,
}

///
/// TODO:
/// 1. multiple bundle
/// 2. dynamic bundle
/// 3. multiple environment process
///
impl<'a> SharedBundle<'a> {
  pub fn new(
    resource_pots: Vec<&'a ResourcePot>,
    module_graph: &'a ModuleGraph,
    context: &'a Arc<CompilationContext>,
  ) -> Result<Self> {
    farm_profile_function!("shared bundle initial");
    let module_analyzer_map: Mutex<HashMap<ModuleId, ModuleAnalyzer>> = Mutex::new(HashMap::new());
    let mut bundle_map: HashMap<ResourcePotId, BundleAnalyzer> = HashMap::new();

    let bundle_variables = Rc::new(RefCell::new(BundleVariable::new()));

    let (toposort_modules, _) = module_graph.toposort();
    let mut order_resource_pot = vec![];
    let order_map: HashMap<ModuleId, usize> = toposort_modules
      .into_iter()
      .enumerate()
      .map(|item| (item.1, item.0))
      .collect();

    // 1. analyze resource pot
    for resource_pot in resource_pots.iter() {
      if !(matches!(
        resource_pot.resource_pot_type,
        ResourcePotType::Js | ResourcePotType::Runtime
      )) {
        continue;
      }
      farm_profile_scope!(format!("analyze resource pot: {:?}", resource_pot.id));

      order_resource_pot.push(resource_pot.id.clone());

      // 1-1. analyze bundle
      let mut bundle_analyzer = BundleAnalyzer::new(
        resource_pot,
        module_graph,
        context,
        bundle_variables.clone(),
      );

      resource_pot
        .modules()
        .into_par_iter()
        .try_for_each(|module_id| {
          let is_dynamic = module_graph.is_dynamic(module_id);
          let is_entry = resource_pot
            .entry_module
            .as_ref()
            .is_some_and(|item| item == module_id);
          let module = module_graph.module(module_id).unwrap();
          let is_runtime = matches!(module.module_type, ModuleType::Runtime);

          // 1-2. analyze bundle module
          let module_analyzer = ModuleAnalyzer::new(
            module,
            context,
            resource_pot.id.clone(),
            is_entry,
            is_dynamic,
            is_runtime,
          )?;

          module_analyzer_map
            .lock()
            .map_c_error()?
            .insert(module_id.clone(), module_analyzer);

          Ok::<(), CompilationError>(())
        })?;

      // 1-3. order bundle module
      bundle_analyzer.build_module_order(&order_map);

      bundle_map.insert(resource_pot.id.clone(), bundle_analyzer);
    }

    // modules manager
    let module_analyzer_manager =
      ModuleAnalyzerManager::new(module_analyzer_map.into_inner().unwrap(), module_graph);

    Ok(Self {
      module_analyzer_manager,
      bundle_map,
      module_graph,
      context,
      bundle_variables,
      order_index_map: order_map,
      order_resource_pot,
    })
  }

  // 2-1 extract module data from ast
  fn extract_modules(&mut self) -> Result<()> {
    farm_profile_function!("");

    for resource_pot_id in &self.order_resource_pot {
      farm_profile_scope!(format!(
        "extract module resource pot: {:?}",
        resource_pot_id
      ));

      let bundle = self
        .bundle_map
        .get_mut(resource_pot_id)
        .map(Ok)
        .unwrap_or_else(|| {
          Err(CompilationError::GenericError(format!(
            "get resource pot {resource_pot_id:?} failed"
          )))
        })?;

      self.module_analyzer_manager.extract_modules_statements(
        &bundle.ordered_modules,
        self.context,
        self.module_graph,
        &mut bundle.bundle_variable.borrow_mut(),
      )?;
    }

    Ok(())
  }

  // 2-2 process common module data
  fn link_modules(&mut self) -> Result<()> {
    farm_profile_function!("");

    let bundle_variable = &mut self.bundle_variables.borrow_mut();

    for resource_pot_id in &self.order_resource_pot {
      bundle_variable.set_namespace(resource_pot_id.clone());

      // polyfill name should make sure it doesn't conflict.
      // tip: but it cannot be rename unresolved mark
      for name in SimplePolyfill::reserved_word() {
        bundle_variable.add_used_name(name);
      }
    }

    self
      .module_analyzer_manager
      .link(bundle_variable, &self.order_index_map, self.context);

    Ok(())
  }

  // 2-3 start process bundle
  fn render_bundle(&mut self) -> Result<()> {
    farm_profile_function!("");

    // TODO: multiple bundle should merge polyfill to runtime bundle, and reexport to other bundle
    for resource_pot_id in &self.order_resource_pot {
      farm_profile_scope!(format!("render bundle: {}", resource_pot_id));

      let bundle_analyzer = self.bundle_map.get_mut(resource_pot_id).unwrap();

      bundle_analyzer.set_namespace(&bundle_analyzer.resource_pot.id);

      bundle_analyzer.render(&mut self.module_analyzer_manager, &self.order_index_map)?;
    }

    let mut polyfill = SimplePolyfill::new(vec![]);

    for resource_pot_id in &self.order_resource_pot {
      let bundle_analyzer = self.bundle_map.get(resource_pot_id).unwrap();

      if matches!(
        bundle_analyzer.resource_pot.resource_pot_type,
        ResourcePotType::Js
      ) {
        polyfill.extends(&bundle_analyzer.polyfill);
      }
    }

    let runtime_resource_pot_id = self.order_resource_pot.iter().find(|item| {
      self.bundle_map.get_mut(*item).is_some_and(|item| {
        matches!(
          item.resource_pot.resource_pot_type,
          ResourcePotType::Runtime
        )
      })
    });

    if let Some(runtime_resource_pot_id) = runtime_resource_pot_id {
      let bundle_analyzer = self.bundle_map.get_mut(runtime_resource_pot_id).unwrap();
      bundle_analyzer.polyfill.extends(&polyfill);
    };

    Ok(())
  }

  // 2. start process bundle
  pub fn render(&mut self) -> Result<()> {
    farm_profile_function!("");

    self.extract_modules()?;

    // TODO: try async foreach
    self.link_modules()?;

    self.render_bundle()?;

    Ok(())
  }

  pub fn codegen(&mut self, resource_pot_id: &String) -> Result<Bundle> {
    farm_profile_function!("");

    let bundle = self.bundle_map.get_mut(resource_pot_id).unwrap();

    let bundle = bundle.codegen(&mut self.module_analyzer_manager, &self.context.config)?;

    Ok(bundle)
  }
}
