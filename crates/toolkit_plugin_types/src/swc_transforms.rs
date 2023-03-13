use farmfe_core::swc_common::Mark;

#[repr(C)]
pub struct FarmSwcTransformReactOptions {
  pub top_level_mark: u32,
  pub unresolved_mark: u32,
  pub inject_helpers: bool,
}

impl Default for FarmSwcTransformReactOptions {
  fn default() -> Self {
    Self {
      top_level_mark: Mark::fresh(Mark::root()).as_u32(),
      unresolved_mark: Mark::fresh(Mark::root()).as_u32(),
      inject_helpers: true,
    }
  }
}
