#[macro_export]
macro_rules! assert_resource_pots {
  ($resource_pots:expr) => {{
    let mut output: Vec<String> = Vec::new();
    for resource_pot in $resource_pots {
      let mut resources = resource_pot.resources();
      let mut module_groups = resource_pot.module_groups.iter().collect::<Vec<_>>();

      resources.sort();
      module_groups.sort_by_key(|a| a.to_string());

      output.push(
        format!(
          "name: {}
id: {}
immutable: {}
resource_pot_type: {:?}
entry: {:#?}
module_groups: {:#?}
modules: {:#?}
resources: {:#?}",
          &resource_pot.name,
          &resource_pot.id,
          resource_pot.immutable,
          &resource_pot.resource_pot_type,
          &resource_pot.entry_module,
          module_groups,
          resource_pot.modules(),
          resources,
        ),
      );

      farmfe_testing_helpers::assert_snapshot!(output.join("\n\n-------\n\n"));
    }
  }};
}

#[macro_export]
macro_rules! assert_sorted_iter_eq {
  ($v:expr) => {{
    let mut left = ($v).iter().cloned().collect::<Vec<_>>();

    left.sort();

    farmfe_testing_helpers::assert_debug_snapshot!(left);
  }};
}
