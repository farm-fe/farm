use std::mem;

use farmfe_core::module::ModuleId;

use crate::resource_pot_to_bundle::uniq_name::BundleVariable;

use super::ModuleAnalyzerManager;

impl BundleVariable {
  pub fn set_uniq_name_both(&mut self, root: usize, local: usize) {
    self.set_var_root(local, root);
    self.set_var_uniq_rename(root);
    self.set_rename(local, self.render_name(root));
  }

  pub fn set_rename_from_other_render_name(&mut self, local: usize, root: usize) {
    self.set_var_root(local, root);
    let rendered_name = self.render_name(root);
    self.set_rename(local, rendered_name);
  }

  pub fn set_rename_from_other_name(&mut self, target: usize, from: usize) {
    self.set_var_root(target, from);
    let name = self.name(from);
    self.set_rename(target, name);
  }

  pub fn set_uniq_name_for_cross_bundle(
    &mut self,
    target: usize,
    local: usize,
    target_id: &ModuleId,
    local_id: &ModuleId,
    module_analyzer_manager: &ModuleAnalyzerManager,
  ) {
    if module_analyzer_manager.is_same_bundle(target_id, local_id) {
      self.set_rename_from_other_name(target, local);
    } else {
      let m = module_analyzer_manager
        .module_analyzer(target_id)
        .map(|i| i.resource_pot_id.to_string())
        .unwrap();
      let prev_namespace = mem::replace(&mut self.namespace, m);

      self.set_var_uniq_rename(target);

      self.set_var_root(local, target);

      self.set_namespace(prev_namespace);

      self.set_var_uniq_rename(local);
    }
  }

  /// modules:
  /// index.js  a.js  b.js
  ///
  /// relation:
  /// index.js -> b.js
  /// b.js -> a.js
  ///
  /// bundle:
  /// index: index.js, a.js
  /// b: b.js
  ///
  /// ```js
  /// // b.js
  /// import { named } from "./a";
  /// export { named as bundleBNamed }
  ///
  /// // index.js
  /// import { bundleBNamed } from "./b";
  /// ```
  ///
  /// 1. index(index.js) import bundleBNamed from b(b.js)
  /// 2. b(b.js) import named from index(a.js)
  ///
  /// step1 should use index bundle var "named", rather than import from b reexport
  ///
  pub fn is_same_bundle_by_root(
    &self,
    index: usize,
    resource_pot_id: &str,
    module_analyzer_manager: &ModuleAnalyzerManager,
  ) -> bool {
    let root = self.var_or_root(index);
    self.module_id_by_var_index(root.index).is_some_and(|m| {
      module_analyzer_manager
        .resource_pot_id(m)
        .is_some_and(|r| r == &resource_pot_id)
    })
  }
}
