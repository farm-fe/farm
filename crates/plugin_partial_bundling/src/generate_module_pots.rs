use std::path::PathBuf;

use farmfe_core::{
  config::partial_bundling::PartialBundlingConfig,
  hashbrown::{HashMap, HashSet},
  module::{module_graph::ModuleGraph, Module, ModuleId},
};
use farmfe_toolkit::resolve::load_package_json;

use crate::{generate_module_buckets::ResourceType, module_pot::ModulePot};

pub fn generate_module_pots(
  modules: &HashSet<ModuleId>,
  module_graph: &ModuleGraph,
  config: &PartialBundlingConfig,
  root: &str,
  resource_type: ResourceType,
) -> Vec<ModulePot> {
  let mut module_pot_map = HashMap::<String, ModulePot>::new();

  for module_id in modules {
    let module = module_graph.module(module_id).unwrap();
    let module_pot_name = generate_module_pot_name(module, config, root, resource_type.clone());
    let module_pot_id = ModulePot::gen_id(
      &module_pot_name,
      module.module_type.clone(),
      module.immutable,
    );

    let module_pot = module_pot_map.entry(module_pot_id).or_insert_with(|| {
      ModulePot::new(
        module_pot_name,
        module.module_type.clone(),
        module.immutable,
      )
    });

    module_pot.add_module(module_id.clone(), module.size, module.execution_order);
  }

  let mut module_pots = module_pot_map
    .into_iter()
    .map(|(_, module_pot)| module_pot)
    .collect::<Vec<_>>();

  module_pots.sort_by_key(|m| m.execution_order);

  module_pots
}

fn generate_module_pot_name(
  module: &Module,
  config: &PartialBundlingConfig,
  root: &str,
  resource_type: ResourceType,
) -> String {
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
        return group_config.name.clone();
      }
    }
  }

  // 2. get name from immutable package
  if module.immutable {
    let package_json = load_package_json(
      PathBuf::from(module.id.resolved_path(root)),
      Default::default(),
    );

    if let Ok(package_json) = package_json {
      let mut name = package_json.name.unwrap_or_default();

      if let Some(version) = package_json.version {
        name = format!("{}@{}", name, version);
      }

      return name;
    }
  }

  module.id.to_string()
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
    },
    hashbrown::HashSet,
    module::{module_graph::ModuleGraph, Module, ModuleType},
  };
  use farmfe_testing_helpers::fixture;

  use crate::{
    generate_module_buckets::ResourceType, generate_module_pots::generate_module_pots,
    module_pot::ModulePot,
  };

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

        module_utils.module_type = ModuleType::Js;
        module_utils.immutable = true;
        module_utils.size = 5 * 1024;
        module_utils.execution_order = 3;

        module_a.module_type = ModuleType::Js;
        module_a.execution_order = 1;
        module_a.immutable = true;
        module_a.size = 1 * 1024;

        module_graph.add_module(module_index);
        module_graph.add_module(module_utils);
        module_graph.add_module(module_a);

        let modules = module_graph
          .modules()
          .iter()
          .map(|m| m.id.clone())
          .collect::<HashSet<_>>();

        let module_pots = generate_module_pots(
          &modules,
          &module_graph,
          &Default::default(),
          cwd.to_str().unwrap(),
          ResourceType::Initial,
        );

        assert_eq!(module_pots.len(), 2);
        assert_eq!(module_pots[0].name, "test-package1@1.0.0");
        assert_eq!(module_pots[0].size, 1 * 1024);
        assert_eq!(module_pots[0].module_type, ModuleType::Js);
        assert_eq!(module_pots[0].immutable, true);
        assert_eq!(module_pots[0].execution_order, 1);
        assert_eq!(
          module_pots[0].modules(),
          &HashSet::from(["tests/fixtures/generate_module_pots/basic1/index.ts".into()])
        );

        assert_eq!(module_pots[1].name, "test-package@1.0.0");
        assert_eq!(module_pots[1].size, 15 * 1024);
        assert_eq!(module_pots[1].module_type, ModuleType::Js);
        assert_eq!(module_pots[1].immutable, true);
        assert_eq!(module_pots[1].execution_order, 2);
        assert_eq!(
          module_pots[1].modules(),
          &HashSet::from([
            "tests/fixtures/generate_module_pots/basic/index.ts".into(),
            "tests/fixtures/generate_module_pots/basic/utils.ts".into()
          ])
        );
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

    let assert_group_works = |module_pots: Vec<ModulePot>| {
      assert_eq!(module_pots.len(), 2);

      assert_eq!(module_pots[0].name, "test");
      assert_eq!(module_pots[0].size, 15 * 1024);
      assert_eq!(module_pots[0].module_type, ModuleType::Js);
      assert_eq!(module_pots[0].immutable, false);
      assert_eq!(module_pots[0].execution_order, 1);
      assert_eq!(
        module_pots[0].modules(),
        &HashSet::from(["src/a.ts".into(), "src/b.ts".into()])
      );

      assert_eq!(module_pots[1].name, "utils/c.ts");
      assert_eq!(module_pots[1].size, 1 * 1024);
      assert_eq!(module_pots[1].module_type, ModuleType::Js);
      assert_eq!(module_pots[1].immutable, false);
      assert_eq!(module_pots[1].execution_order, 3);
      assert_eq!(
        module_pots[1].modules(),
        &HashSet::from(["utils/c.ts".into()])
      );
    };

    let assert_group_not_works = |module_pots: Vec<ModulePot>| {
      assert_eq!(module_pots.len(), 3);

      assert_eq!(module_pots[0].name, "src/a.ts");
      assert_eq!(module_pots[0].size, 10 * 1024);
      assert_eq!(module_pots[0].module_type, ModuleType::Js);
      assert_eq!(module_pots[0].immutable, false);
      assert_eq!(module_pots[0].execution_order, 1);
      assert_eq!(
        module_pots[0].modules(),
        &HashSet::from(["src/a.ts".into()])
      );

      assert_eq!(module_pots[1].name, "src/b.ts");
      assert_eq!(module_pots[1].size, 5 * 1024);
      assert_eq!(module_pots[1].module_type, ModuleType::Js);
      assert_eq!(module_pots[1].immutable, false);
      assert_eq!(module_pots[1].execution_order, 2);
      assert_eq!(
        module_pots[1].modules(),
        &HashSet::from(["src/b.ts".into()])
      );

      assert_eq!(module_pots[2].name, "utils/c.ts");
      assert_eq!(module_pots[2].size, 1 * 1024);
      assert_eq!(module_pots[2].module_type, ModuleType::Js);
      assert_eq!(module_pots[2].immutable, false);
      assert_eq!(module_pots[2].execution_order, 3);
      assert_eq!(
        module_pots[2].modules(),
        &HashSet::from(["utils/c.ts".into()])
      );
    };

    // Default config for group_type and resource_type
    let config = PartialBundlingConfig {
      groups: vec![PartialBundlingGroupConfig {
        name: "test".into(),
        test: vec![ConfigRegex::new("src/.*")],
        ..Default::default()
      }],
      ..Default::default()
    };

    let module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      "/root",
      ResourceType::Initial,
    );
    assert_group_works(module_pots);

    // only match mutable modules
    let config = PartialBundlingConfig {
      groups: vec![PartialBundlingGroupConfig {
        name: "test".into(),
        test: vec![ConfigRegex::new("src/.*")],
        group_type: PartialBundlingGroupConfigGroupType::Immutable,
        ..Default::default()
      }],
      ..Default::default()
    };

    let module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      "/root",
      ResourceType::Initial,
    );

    assert_group_not_works(module_pots);

    let config = PartialBundlingConfig {
      groups: vec![PartialBundlingGroupConfig {
        name: "test".into(),
        test: vec![ConfigRegex::new("src/.*")],
        resource_type: PartialBundlingGroupConfigResourceType::Async,
        ..Default::default()
      }],
      ..Default::default()
    };

    let module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      "/root",
      ResourceType::Initial,
    );

    assert_group_not_works(module_pots);

    let module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      "/root",
      ResourceType::Async,
    );

    assert_group_works(module_pots);

    let config = PartialBundlingConfig {
      groups: vec![PartialBundlingGroupConfig {
        name: "test".into(),
        test: vec![ConfigRegex::new("src/.*")],
        resource_type: PartialBundlingGroupConfigResourceType::Initial,
        ..Default::default()
      }],
      ..Default::default()
    };

    let module_pots = generate_module_pots(
      &modules,
      &module_graph,
      &config,
      "/root",
      ResourceType::Initial,
    );

    assert_group_works(module_pots);
  }
}
