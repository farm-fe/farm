use crate::resource_pot_to_bundle::uniq_name::BundleVariable;

impl BundleVariable {
  pub fn set_uniq_name_both(&mut self, v1: usize, v2: usize) {
    self.set_var_uniq_rename(v1);
    self.set_rename(v2, self.render_name(v1));
  }

  pub fn set_rename_from_other_render_name(&mut self, target: usize, from: usize) {
    self.set_rename(target, self.render_name(from));
  }

  pub fn set_rename_from_other_name(&mut self, target: usize, from: usize) {
    self.set_rename(target, self.name(from));
  }
}
