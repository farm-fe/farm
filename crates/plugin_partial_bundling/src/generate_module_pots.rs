use std::collections::{HashMap, HashSet};

use farmfe_core::{
  config::{partial_bundling::PartialBundlingConfig, Config},
  module::{module_graph::ModuleGraph, Module, ModuleId},
};

use crate::{
  generate_module_buckets::ResourceType, module_pot::ModulePot, utils::group_is_enforce,
};

pub fn generate_module_pots(
  modules: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  config: &Config,
  resource_type: ResourceType,
  groups_enforce_map: &HashMap<String, bool>,
) -> Vec<ModulePot> {
  let partial_bundling = &config.partial_bundling;
  let mut module_pot_map = HashMap::<String, ModulePot>::new();

  for module_id in modules {
    let module = module_graph.module(module_id).unwrap();
    let module_pot_meta = generate_module_pot_meta(
      module,
      partial_bundling,
      resource_type.clone(),
      groups_enforce_map,
    );
    let module_pot_id = ModulePot::gen_id(
      &module_pot_meta.id,
      module.module_type.clone(),
      module.immutable,
    );

    let module_pot = module_pot_map.entry(module_pot_id).or_insert_with(|| {
      ModulePot::new(
        module_pot_meta.id,
        module_pot_meta.name,
        module.module_type.clone(),
        module.immutable,
        module_pot_meta.enforce,
      )
    });

    module_pot.add_module(module_id.clone(), module.size, module.execution_order);
  }

  // split module_pots from module_pot_map that its size larger that target_max_size
  let mut exceed_size_module_pot_ids = module_pot_map
    .iter()
    .filter(|(_, module_pot)| module_pot.size > partial_bundling.target_max_size)
    .map(|(module_pot_id, _)| module_pot_id.clone())
    .collect::<Vec<_>>();
  exceed_size_module_pot_ids.sort();

  for exceed_size_module_pot_id in exceed_size_module_pot_ids {
    let module_pot = module_pot_map.remove(&exceed_size_module_pot_id).unwrap();
    let new_module_pot_numbers = (module_pot.size / partial_bundling.target_max_size) + 1;
    let module_pot_name = module_pot.name.clone();
    let module_pot_id = module_pot.id.clone();
    let immutable = module_pot.immutable;
    let ty = module_pot.module_type.clone();
    let enforce = module_pot.enforce;
    let mut modules = module_pot.take_modules().into_iter().collect::<Vec<_>>();
    modules.sort_by_key(|m| m.to_string());
    let page_size = modules.len() / new_module_pot_numbers;

    for i in 0..new_module_pot_numbers {
      let new_module_pot_name = format!("{}-{}", module_pot_id, i);
      let new_module_pot_id = ModulePot::gen_id(&new_module_pot_name, ty.clone(), immutable);

      let new_module_pot = module_pot_map.entry(new_module_pot_id).or_insert_with(|| {
        ModulePot::new(
          new_module_pot_name,
          module_pot_name.clone(),
          ty.clone(),
          immutable,
          enforce,
        )
      });

      let start = i * page_size;
      let end = if i == new_module_pot_numbers - 1 {
        modules.len()
      } else {
        (i + 1) * page_size
      };

      for module_id in &modules[start..end] {
        let module = module_graph.module(module_id).unwrap();
        new_module_pot.add_module(module_id.clone(), module.size, module.execution_order);
      }
    }
  }

  let mut module_pots = module_pot_map
    .into_iter()
    .map(|(_, module_pot)| module_pot)
    .collect::<Vec<_>>();

  module_pots.sort_by_key(|m| m.execution_order);

  module_pots
}

#[derive(Debug, Default)]
pub struct ModulePotMeta {
  id: String,
  name: Option<String>,
  enforce: bool,
}

fn generate_module_pot_meta(
  module: &Module,
  config: &PartialBundlingConfig,
  resource_type: ResourceType,
  groups_enforce_map: &HashMap<String, bool>,
) -> ModulePotMeta {
  // 1. get name from partialBundling.groups
  for group_config in &config.groups {
    // use the first matched group name, so the order of groups is important
    if group_config
      .test
      .iter()
      .any(|c| c.is_match(&module.id.to_string()))
    {
      if (group_config.group_type.is_match(module.immutable))
        && (resource_type.is_match(group_config.resource_type.clone()))
      {
        return ModulePotMeta {
          name: Some(group_config.name.clone()),
          id: group_config.name.clone(),
          enforce: group_is_enforce(&group_config.name, groups_enforce_map),
        };
      }
    }
  }

  // 2. get name from immutable package
  if module.immutable {
    return ModulePotMeta {
      id: format!("{}@{}", module.package_name, module.package_version),
      ..Default::default()
    };
  }

  ModulePotMeta {
    id: module.id.to_string(),
    name: None,
    enforce: false,
  }
}

#[cfg(test)]
mod tests {
  use farmfe_core::{
    config::{
      config_regex::ConfigRegex,
      partial_bundling::{
        PartialBundlingConfig, PartialBundlingGroupConfig, PartialBundlingGroupConfigGroupType,
        PartialBundlingGroupConfigResourceType,
      },
      Config,
    },
    module::{module_graph::ModuleGraph, Module, ModuleType},
  };
  use farmfe_testing_helpers::{assert_debug_snapshot, fixture};
  use std::collections::HashSet;
  use std::mem;

  use crate::{generate_module_buckets::ResourceType, generate_module_pots::generate_module_pots};

  macro_rules! assert_module_pot_snapshot {
    ($module_pots:expr) => {
      for module_pot in $module_pots.iter_mut() {
        let mut modules = mem::take(&mut module_pot.modules)
          .into_iter()
          .collect::<Vec<_>>();
        modules.sort();
        assert_debug_snapshot!((&module_pot, &modules));
        module_pot.modules = modules.into_iter().collect();
      }
    };
  }

  #[test]
  fn test_generate_module_pots_package() {
    fixture!(
      "tests/fixtures/generate_module_pots/basic/index.ts",
      |_, cwd| {
        let mut module_graph = ModuleGraph::new();
        let mut module_index =
          Module::new("tests/fixtures/generate_module_pots/basic/index.ts".into());
        let mut module_utils =
          Module::new("tests/fixtures/generate_module_pots/basic/utils.ts".into());
        let mut module_a =
          Module::new("tests/fixtures/generate_module_pots/basic1/index.ts".into());

        module_index.module_type = ModuleType::Js;
        module_index.immutable = true;
        module_index.size = 10 * 1024;
        module_index.execution_order = 2;
        module_index.package_name = "test-package".to_string();
        module_index.package_version = "1.0.0".to_string();

        module_utils.module_type = ModuleType::Js;
        module_utils.immutable = true;
        module_utils.size = 5 * 1024;
        module_utils.execution_order = 3;
        module_utils.package_name = "test-package".to_string();
        module_utils.package_version = "1.0.0".to_string();

        module_a.module_type = ModuleType::Js;
        module_a.execution_order = 1;
        module_a.immutable = true;
        module_a.size = 1 * 1024;
        module_a.package_name = "test-package1".to_string();
        module_a.package_version = "1.0.0".to_string();

        module_graph.add_module(module_index);
        module_graph.add_module(module_utils);
        module_graph.add_module(module_a);

        let modules = module_graph
          .modules()
          .iter()
          .map(|m| m.id.clone())
          .collect::<HashSet<_>>();

        let mut module_pots = generate_module_pots(
          &modules,
          &module_graph,
          &Default::default(),
          ResourceType::Initial,
          &Default::default(),
        );

        assert_eq!(module_pots.len(), 2);
        assert_module_pot_snapshot!(module_pots);
        // assert_eq!(module_pots[0].name, "test-package1@1.0.0");
        // assert_eq!(module_pots[0].size, 1 * 1024);
        // assert_eq!(module_pots[0].module_type, ModuleType::Js);
        // assert_eq!(module_pots[0].immutable, true);
        // assert_eq!(module_pots[0].execution_order, 1);
        // assert_eq!(
        //   module_pots[0].modules(),
        //   &HashSet::from(["tests/fixtures/generate_module_pots/basic1/index.ts".into()])
        // );

        // assert_eq!(module_pots[1].name, "test-package@1.0.0");
        // assert_eq!(module_pots[1].size, 15 * 1024);
        // assert_eq!(module_pots[1].module_type, ModuleType::Js);
        // assert_eq!(module_pots[1].immutable, true);
        // assert_eq!(module_pots[1].execution_order, 2);
        // assert_eq!(
        //   module_pots[1].modules(),
        //   &HashSet::from([
        //     "tests/fixtures/generate_module_pots/basic/index.ts".into(),
        //     "tests/fixtures/generate_module_pots/basic/utils.ts".into()
        //   ])
        // );
      }
    );
  }

  #[test]
  fn test_generate_module_pots_group() {
    let mut module_graph = ModuleGraph::new();
    let mut module_a = Module::new("src/a.ts".into());
    let mut module_b = Module::new("src/b.ts".into());
    let mut module_c = Module::new("utils/c.ts".into());

    module_a.module_type = ModuleType::Js;
    module_a.size = 10 * 1024;
    module_a.execution_order = 1;

    module_b.module_type = ModuleType::Js;
    module_b.size = 5 * 1024;
    module_b.execution_order = 2;

    module_c.module_type = ModuleType::Js;
    module_c.size = 1 * 1024;
    module_c.execution_order = 3;

    module_graph.add_module(module_a);
    module_graph.add_module(module_b);
    module_graph.add_module(module_c);

    let modules = module_graph
      .modules()
      .iter()
      .map(|m| m.id.clone())
      .collect::<HashSet<_>>();

    // Default config for group_type and resource_type
    let config = Config {
      partial_bundling: Box::new(PartialBundlingConfig {
        groups: vec![PartialBundlingGroupConfig {
          name: "test".into(),
          test: vec![ConfigRegex::new("src/.*")],
          ..Default::default()
        }],
        ..Default::default()
      }),
      ..Default::default()
    };

    let mut module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      ResourceType::Initial,
      &Default::default(),
    );
    assert_module_pot_snapshot!(module_pots);

    // only match mutable modules
    let config = Config {
      partial_bundling: Box::new(PartialBundlingConfig {
        groups: vec![PartialBundlingGroupConfig {
          name: "test".into(),
          test: vec![ConfigRegex::new("src/.*")],
          group_type: PartialBundlingGroupConfigGroupType::Immutable,
          ..Default::default()
        }],
        ..Default::default()
      }),
      ..Default::default()
    };

    let mut module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      ResourceType::Initial,
      &Default::default(),
    );
    assert_module_pot_snapshot!(module_pots);

    let config = Config {
      partial_bundling: Box::new(PartialBundlingConfig {
        groups: vec![PartialBundlingGroupConfig {
          name: "test".into(),
          test: vec![ConfigRegex::new("src/.*")],
          resource_type: PartialBundlingGroupConfigResourceType::Async,
          ..Default::default()
        }],
        ..Default::default()
      }),
      ..Default::default()
    };

    let mut module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      ResourceType::Initial,
      &Default::default(),
    );
    assert_module_pot_snapshot!(module_pots);

    let mut module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      ResourceType::Async,
      &Default::default(),
    );
    assert_module_pot_snapshot!(module_pots);

    let config = Config {
      partial_bundling: Box::new(PartialBundlingConfig {
        groups: vec![PartialBundlingGroupConfig {
          name: "test".into(),
          test: vec![ConfigRegex::new("src/.*")],
          resource_type: PartialBundlingGroupConfigResourceType::Initial,
          ..Default::default()
        }],
        ..Default::default()
      }),
      ..Default::default()
    };

    let mut module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      ResourceType::Initial,
      &Default::default(),
    );
    assert_module_pot_snapshot!(module_pots);
  }
}
