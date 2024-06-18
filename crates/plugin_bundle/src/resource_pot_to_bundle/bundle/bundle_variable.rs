use crate::resource_pot_to_bundle::uniq_name::BundleVariable;

impl BundleVariable {
  pub fn set_uniq_name_both(&mut self, v1: usize, v2: usize) {
    self.set_var_uniq_rename(v1);
    self.set_rename(v2, self.render_name(v1));
  }

  pub fn set_rename_from_other_render_name(&mut self, target: usize, from: usize) {
    let rendered_name = self.render_name(from);
    self.set_rename(target, rendered_name);
  }

  pub fn set_rename_from_other_name(&mut self, target: usize, from: usize) {
    let name = self.name(from);
    self.set_rename(target, name);
  }
}
