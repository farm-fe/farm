#[macro_export]
macro_rules! assert_resource_pots {
  ($resource_pots:expr) => {
    for resource_pot in $resource_pots {
      let mut resources = resource_pot.resources();
      let mut module_groups = resource_pot.module_groups.iter().collect::<Vec<_>>();

      resources.sort();
      module_groups.sort_by_key(|a| a.to_string());

      farmfe_testing_helpers::assert_debug_snapshot!((
        ("immutable", resource_pot.immutable),
        ("modules", resource_pot.modules()),
        ("resource_pot_type", &resource_pot.resource_pot_type),
        ("name", &resource_pot.name),
        ("id", &resource_pot.id),
        ("entry", &resource_pot.entry_module),
        ("resources", resources),
        ("module_groups", module_groups),
      ));
    }
  };
}

#[macro_export]
macro_rules! assert_sorted_iter_eq {
  ($v:expr) => {
    let mut left = ($v).iter().cloned().collect::<Vec<_>>();

    left.sort();

    farmfe_testing_helpers::assert_debug_snapshot!(left);
  };
}
