use std::{
  cell::RefCell,
  collections::HashMap,
  hash::Hash,
  rc::Rc,
  sync::{Arc, Mutex},
};

use bundle::bundle_reference::BundleReferenceManager;
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
pub use common::{FARM_BUNDLE_POLYFILL_SLOT, FARM_BUNDLE_REFERENCE_SLOT_PREFIX};

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Var {
  var: Id,
  rename: Option<String>,
  removed: bool,
  root: Option<usize>,
  module_id: Option<usize>,
  index: usize,
  // only for uniq name
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

mod polyfill;
mod uniq_name;

pub type BundleMap<'a> = HashMap<ResourcePotId, BundleAnalyzer<'a>>;

pub struct SharedBundle<'a> {
  pub bundle_map: BundleMap<'a>,
  bundle_reference: BundleReferenceManager,
  module_analyzer_manager: ModuleAnalyzerManager<'a>,
  module_graph: &'a ModuleGraph,
  context: &'a Arc<CompilationContext>,
  pub bundle_variables: Rc<RefCell<BundleVariable>>,
  order_index_map: Arc<HashMap<ModuleId, usize>>,
  order_resource_pot: Vec<ResourcePotId>,
}

///
/// TODO:
/// 1. multiple environment process
///   - browser polyfill
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

    let (toposort_modules, circles) = module_graph.toposort();
    println!("circles: {:#?}", circles);
    let mut order_resource_pot = vec![];
    let order_map: Arc<HashMap<ModuleId, usize>> = Arc::new(
      toposort_modules
        .iter()
        .enumerate()
        .map(|item| (item.1.clone(), item.0.clone()))
        .collect(),
    );

    bundle_variables.borrow_mut().module_order_map = order_map.clone();
    bundle_variables.borrow_mut().module_order_index_set = Arc::new(toposort_modules);

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

    let bundle_reference_manager = BundleReferenceManager::default();

    Ok(Self {
      module_analyzer_manager,
      bundle_map,
      module_graph,
      context,
      bundle_variables,
      order_index_map: order_map,
      order_resource_pot,
      bundle_reference: bundle_reference_manager,
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
  fn link_resource_polyfill_to_variables(&mut self) {
    let bundle_variable = &mut self.bundle_variables.borrow_mut();

    let polyfill_module_id = ModuleId::from(FARM_BUNDLE_POLYFILL_SLOT);

    let mut reserved_word = SimplePolyfill::reserved_word();

    reserved_word.push("module".to_string());

    if let Some(bundle_analyzer) = self
      .module_analyzer_manager
      .module_analyzer(&polyfill_module_id)
      .and_then(|r| self.bundle_map.get_mut(&r.resource_pot_id))
    {
      bundle_variable.set_namespace(bundle_analyzer.resource_pot.id.clone());

      for name in &reserved_word {
        let var = bundle_variable.register_var(&polyfill_module_id, &name.as_str().into(), false);
        bundle_variable.polyfill_index_map.insert(name.clone(), var);
      }
    };

    for resource_pot_id in &self.order_resource_pot {
      bundle_variable.set_namespace(resource_pot_id.clone());

      // polyfill name should make sure it doesn't conflict.
      // tip: but it cannot be rename unresolved mark
      for name in &reserved_word {
        bundle_variable.add_used_name(name.clone());
      }
    }
  }

  // 2-3
  fn link_modules_meta(&mut self) -> Result<()> {
    farm_profile_function!("");

    self.module_analyzer_manager.link(
      &mut self.bundle_variables.borrow_mut(),
      &self.order_index_map,
      self.context,
      &self.order_resource_pot,
    );

    Ok(())
  }

  // 2-4
  fn render_bundle(&mut self) -> Result<()> {
    farm_profile_function!("");

    self.each_render()?;

    self.each_patch_ast()?;

    self.patch_polyfill()?;

    Ok(())
  }

  fn each_render(&mut self) -> Result<()> {
    // let mut defer_bundle_relation = vec![];

    let mut ordered_modules = self.order_index_map.iter().collect::<Vec<_>>();

    ordered_modules.sort_by(|a, b| a.1.cmp(b.1).reverse());

    for resource_pot_id in &self.order_resource_pot {
      farm_profile_scope!(format!("render bundle: {}", resource_pot_id));

      let bundle_analyzer = self.bundle_map.get_mut(resource_pot_id).unwrap();

      bundle_analyzer.set_namespace(&bundle_analyzer.resource_pot.id);

      bundle_analyzer.render(&mut self.module_analyzer_manager)?;
    }

    for (module_id, _) in ordered_modules {
      println!("module_id: {}", module_id.to_string());
      farm_profile_scope!(format!("render module: {}", module_id));

      let Some(module_analyzer) = self.module_analyzer_manager.module_analyzer(module_id) else {
        continue;
      };

      let resource_pot_id = module_analyzer.resource_pot_id.clone();

      let bundle_analyzer = self.bundle_map.get_mut(&resource_pot_id).unwrap();

      bundle_analyzer.set_namespace(&bundle_analyzer.resource_pot.id);

      bundle_analyzer.link_module_relation(
        module_id,
        &mut self.module_analyzer_manager,
        &mut self.bundle_reference,
      )?;
    }

    Ok(())
  }

  fn each_patch_ast(&mut self) -> Result<()> {
    for resource_pot_id in &self.order_resource_pot {
      let bundle_analyzer = self.bundle_map.get_mut(resource_pot_id).unwrap();

      bundle_analyzer.set_namespace(&bundle_analyzer.resource_pot.id);

      bundle_analyzer.patch_ast(
        &mut self.module_analyzer_manager,
        &self.order_index_map,
        &mut self.bundle_reference,
      )?;
    }

    Ok(())
  }

  fn patch_polyfill(&mut self) -> Result<()> {
    // multiple bundle should merge polyfill to runtime or entry bundle, and reexport to other bundle
    let mut polyfill = SimplePolyfill::new(vec![]);

    let polyfill_resource_pot = self.module_analyzer_manager.polyfill_resource_pot();
    for resource_pot_id in &self.order_resource_pot {
      let bundle_analyzer = self.bundle_map.get_mut(resource_pot_id).unwrap();

      if polyfill_resource_pot
        .as_ref()
        .is_some_and(|r| resource_pot_id != r)
      {
        bundle_analyzer.patch_polyfill_for_bundle(&mut self.module_analyzer_manager)?;
      }

      if matches!(
        bundle_analyzer.resource_pot.resource_pot_type,
        ResourcePotType::Js
      ) {
        polyfill.extends(&bundle_analyzer.polyfill);
      }
    }

    let polyfill_bundle = self.bundle_map.values_mut().find(|item| {
      let module_id = ModuleId::from(FARM_BUNDLE_POLYFILL_SLOT);
      item.ordered_modules.contains(&&module_id)
    });

    if let Some(bundle_analyzer) = polyfill_bundle {
      bundle_analyzer.patch_polyfill(&mut self.module_analyzer_manager, polyfill)?;
    };

    Ok(())
  }

  // 2. start process bundle
  pub fn render(&mut self) -> Result<()> {
    farm_profile_function!("");

    self.link_resource_polyfill_to_variables();

    self.extract_modules()?;

    // TODO: try async foreach
    self.link_modules_meta()?;

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
