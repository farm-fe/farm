use std::sync::Arc;

use farmfe_core::{
  config::ModuleFormat,
  context::CompilationContext,
  enhanced_magic_string::bundle::Bundle,
  module::{module_graph::ModuleGraph, ModuleId},
  resource::resource_pot::{ResourcePot, ResourcePotType},
  HashMap, HashSet,
};

use farmfe_plugin_bundle::resource_pot_to_bundle::{BundleGroup, ShareBundleOptions, SharedBundle};

/// Note: Scope Hoisting is enabled only `config.concatenate_modules` is true. Otherwise, it A module is a [ScopeHoistedModuleGroup]
///
/// The [ModuleId]s that can be hoisted into the same Module. For example:
/// ```md
///         A    F
///        / \  /
///       B   C
///      / \
///     D   E
/// ```
/// The [ModuleId]s of `A`, `B`, `D`, `E` can be hoisted into the same Module `A`. But `C` cannot cause C has 2 independencies.

#[derive(Debug, PartialEq, Eq)]
pub struct ScopeHoistedModuleGroup {
  /// The [ModuleId] that other modules hoisted to, it's the entry of this [ScopeHoistedModuleGroup].
  pub target_hoisted_module_id: ModuleId,
  /// The [ModuleId]s that this [ScopeHoistedModuleGroup] hoisted to. Include the [target_hoisted_module_id].
  pub hoisted_module_ids: HashSet<ModuleId>,
}

impl ScopeHoistedModuleGroup {
  pub fn new(target_hoisted_module_id: ModuleId) -> Self {
    Self {
      hoisted_module_ids: HashSet::from_iter([target_hoisted_module_id.clone()]),
      target_hoisted_module_id,
    }
  }

  pub fn extend_hoisted_module_ids(&mut self, hoisted_module_ids: HashSet<ModuleId>) {
    self.hoisted_module_ids.extend(hoisted_module_ids);
  }

  /// Render this [ScopeHoistedModuleGroup] to a Farm runtime module. For example:
  /// ```js
  /// function(module, exports, farmRequire, farmDynamicRequire) {
  ///   const xxx = farmDynamicRequire('./xxx');
  ///
  ///   const module_D = 'D'; // hoisted code of module D
  ///   const module_C = 'C'; // hoisted code of module C
  ///   const module_B = 'B'; // hoisted code of module B
  ///   console.log(module_D, module_C, module_B, xxx); // code of module A
  ///
  ///   module.o(exports, 'b', module_B);
  /// }
  /// ```
  pub fn render(
    &self,
    module_graph: &ModuleGraph,
    context: &Arc<CompilationContext>,
  ) -> farmfe_core::error::Result<Bundle> {
    let bundle_id = self.target_hoisted_module_id.to_string();

    let mut share_bundle = SharedBundle::new(
      vec![BundleGroup {
        id: bundle_id.clone(),
        modules: self.hoisted_module_ids.iter().collect(),
        entry_module: Some(self.target_hoisted_module_id.clone()),
        group_type: ResourcePotType::Js,
      }],
      module_graph,
      context,
      Some(ShareBundleOptions {
        reference_slot: false,
        ignore_external_polyfill: true,
        // should ignore
        format: ModuleFormat::EsModule,
        hash_path: true,
        concatenation_module: true,
        ..Default::default()
      }),
    )?;

    share_bundle.render()?;
    share_bundle.codegen(&bundle_id)
  }
}

/// Handle the modules of a resource pot in topological order.
/// Merge the modules into a [ScopeHoistedModuleGroup] if all of the dependents of that module are in the same [ScopeHoistedModuleGroup].
///
/// Note: A module is a [ScopeHoistedModuleGroup] if config.concatenate_modules is false.
pub fn build_scope_hoisted_module_groups(
  resource_pot: &ResourcePot,
  module_graph: &ModuleGraph,
  context: &Arc<CompilationContext>,
) -> Vec<ScopeHoistedModuleGroup> {
  let mut scope_hoisted_module_groups_map = HashMap::default();
  let mut reverse_module_hoisted_group_map = HashMap::default();

  for module_id in resource_pot.modules() {
    scope_hoisted_module_groups_map.insert(
      module_id.clone(),
      ScopeHoistedModuleGroup::new(module_id.clone()),
    );
    reverse_module_hoisted_group_map.insert(module_id.clone(), module_id.clone());
  }
  // println!(
  //   "scope_hoisted_module_groups_map: {:?}",
  //   scope_hoisted_module_groups_map
  // );
  // Merge ScopeHoistedModuleGroup when concatenate_modules enabled
  if context.config.concatenate_modules {
    let mut scope_hoisted_module_groups = scope_hoisted_module_groups_map
      .values()
      .collect::<Vec<&ScopeHoistedModuleGroup>>();
    // 1. topological sort
    scope_hoisted_module_groups.sort_by(|a, b| {
      let ma = module_graph.module(&a.target_hoisted_module_id).unwrap();
      let mb = module_graph.module(&b.target_hoisted_module_id).unwrap();
      // larger execution_order means it's the importer
      mb.execution_order.cmp(&ma.execution_order)
    });

    let mut merged_scope_hoisted_module_groups_map: HashMap<ModuleId, HashSet<ModuleId>> =
      HashMap::default();

    for group in scope_hoisted_module_groups {
      let module = module_graph
        .module(&group.target_hoisted_module_id)
        .unwrap();
      // if this module is not an esm module, skip it
      if !module.meta.as_script().is_esm() {
        continue;
      }

      let dependents = module_graph.dependents_ids(&group.target_hoisted_module_id);
      // there dependents of this module are not in this resource pot
      if dependents.iter().any(|id| {
        !resource_pot.has_module(id)
          || !module_graph
            .module(id)
            .is_some_and(|m| m.meta.as_script().is_esm())
      }) {
        continue;
      }

      let dependents_hoisted_group_ids = dependents
        .into_iter()
        .map(|id| reverse_module_hoisted_group_map.get(&id).unwrap().clone())
        .collect::<HashSet<ModuleId>>();

      // all of the dependents of this module are in the same [ScopeHoistedModuleGroup]
      if dependents_hoisted_group_ids.len() == 1 {
        let dependents_hoisted_group_id = dependents_hoisted_group_ids.into_iter().next().unwrap();

        // if execution_order of dependents_hoisted_group_id is smaller than this module, means there is a cycle, skip it
        let dependents_hoisted_group_module =
          module_graph.module(&dependents_hoisted_group_id).unwrap();

        if module_graph
          .circle_record
          .is_in_circle(&dependents_hoisted_group_id)
          || dependents_hoisted_group_module.execution_order
            < module_graph
              .module(&group.target_hoisted_module_id)
              .unwrap()
              .execution_order
        {
          continue;
        }

        let merged_map = merged_scope_hoisted_module_groups_map
          .entry(dependents_hoisted_group_id.clone())
          .or_default();
        merged_map.insert(group.target_hoisted_module_id.clone());

        for hoisted_module_id in &group.hoisted_module_ids {
          reverse_module_hoisted_group_map.insert(
            hoisted_module_id.clone(),
            dependents_hoisted_group_id.clone(),
          );
        }
      }
    }

    for (target_hoisted_module_id, hoisted_module_ids) in
      merged_scope_hoisted_module_groups_map.into_iter()
    {
      let mut all_hoisted_module_ids = HashSet::default();

      for hoisted_module_id in hoisted_module_ids {
        let hoisted_module_group = scope_hoisted_module_groups_map
          .remove(&hoisted_module_id)
          .unwrap();
        all_hoisted_module_ids.extend(hoisted_module_group.hoisted_module_ids);
      }

      let target_hoisted_module_group = scope_hoisted_module_groups_map
        .get_mut(&target_hoisted_module_id)
        .unwrap();

      target_hoisted_module_group.extend_hoisted_module_ids(all_hoisted_module_ids);
    }
  }

  let mut res = scope_hoisted_module_groups_map
    .into_values()
    .collect::<Vec<ScopeHoistedModuleGroup>>();
  res.sort_by_key(|group| group.target_hoisted_module_id.to_string());

  res
}

#[cfg(test)]
mod tests {
  use farmfe_core::{
    config::Config,
    context::CompilationContext,
    resource::resource_pot::{ResourcePot, ResourcePotType},
    HashSet,
  };
  use farmfe_testing_helpers::construct_test_module_graph;

  #[test]
  fn test_build_scope_hoisted_module_groups() {
    let mut module_graph = construct_test_module_graph();
    module_graph.update_execution_order_for_modules();

    let mut resource_pot = ResourcePot::new("test".to_string(), ResourcePotType::Js);

    for module in module_graph.modules() {
      resource_pot.add_module(module.id.clone());
    }

    let context = CompilationContext::new(
      Config {
        concatenate_modules: true,
        ..Default::default()
      },
      vec![],
    )
    .unwrap();

    let scope_hoisted_module_groups = super::build_scope_hoisted_module_groups(
      &resource_pot,
      &module_graph,
      &std::sync::Arc::new(context),
    );

    println!("{:#?}", scope_hoisted_module_groups);
    // groups: (A, C), (B, E, G), (F), (D)
    assert_eq!(
      scope_hoisted_module_groups,
      vec![
        super::ScopeHoistedModuleGroup {
          target_hoisted_module_id: "A".into(),
          hoisted_module_ids: HashSet::from_iter(["A".into()]),
        },
        super::ScopeHoistedModuleGroup {
          target_hoisted_module_id: "B".into(),
          hoisted_module_ids: HashSet::from_iter(["B".into(), "E".into(), "G".into(),]),
        },
        super::ScopeHoistedModuleGroup {
          target_hoisted_module_id: "C".into(),
          hoisted_module_ids: HashSet::from_iter(["C".into()]),
        },
        super::ScopeHoistedModuleGroup {
          target_hoisted_module_id: "D".into(),
          hoisted_module_ids: HashSet::from_iter(["D".into(),]),
        },
        super::ScopeHoistedModuleGroup {
          target_hoisted_module_id: "F".into(),
          hoisted_module_ids: HashSet::from_iter(["F".into(),]),
        },
      ]
    );
  }
}
