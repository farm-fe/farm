use serde_json::Value;

/// Bitflags that describe which Tailwind CSS features are used in the compiled
/// CSS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Features(u32);

impl Features {
  pub const NONE: Self = Self(0);
  pub const AT_APPLY: Self = Self(1 << 0);
  pub const JS_PLUGIN_COMPAT: Self = Self(1 << 1);
  pub const THEME_FUNCTION: Self = Self(1 << 2);
  pub const UTILITIES: Self = Self(1 << 3);

  pub fn contains(self, other: Self) -> bool {
    self.0 & other.0 != 0
  }

  pub fn has_any_output_feature(self) -> bool {
    self.contains(Self::AT_APPLY)
      || self.contains(Self::JS_PLUGIN_COMPAT)
      || self.contains(Self::THEME_FUNCTION)
      || self.contains(Self::UTILITIES)
  }
}

impl std::ops::BitOr for Features {
  type Output = Self;
  fn bitor(self, rhs: Self) -> Self {
    Self(self.0 | rhs.0)
  }
}

impl std::ops::BitOrAssign for Features {
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

impl std::ops::BitAnd for Features {
  type Output = Self;
  fn bitand(self, rhs: Self) -> Self {
    Self(self.0 & rhs.0)
  }
}

/// Polyfill flags that control which CSS compatibility transforms are applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Polyfills(u32);

impl Polyfills {
  pub const NONE: Self = Self(0);
  pub const AT_MEDIA_HOVER: Self = Self(1 << 0);

  pub fn contains(self, other: Self) -> bool {
    self.0 & other.0 != 0
  }
}

impl std::ops::BitOr for Polyfills {
  type Output = Self;
  fn bitor(self, rhs: Self) -> Self {
    Self(self.0 | rhs.0)
  }
}

impl std::ops::BitOrAssign for Polyfills {
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

/// Externally supplied Tailwind config payload.
///
/// This crate accepts the config as data and does not load JS config files.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TailwindConfig {
  pub data: Value,
}

impl TailwindConfig {
  pub fn new(data: Value) -> Self {
    Self { data }
  }
}

/// Options passed to [`compile`].
#[derive(Debug, Clone)]
pub struct CompilerOptions {
  pub features: Features,
  pub polyfills: Polyfills,
  pub dependencies: Vec<String>,
  pub source_maps_enabled: bool,
  pub config: Option<TailwindConfig>,
}

impl Default for CompilerOptions {
  fn default() -> Self {
    Self {
      features: Features::NONE,
      polyfills: Polyfills::NONE,
      dependencies: Vec::new(),
      source_maps_enabled: false,
      config: None,
    }
  }
}

/// Compiled core state.
#[derive(Debug, Clone)]
pub struct Compiler {
  css: String,
  pub features: Features,
  pub polyfills: Polyfills,
  dependencies: Vec<String>,
  source_maps_enabled: bool,
  config: Option<TailwindConfig>,
}

impl Compiler {
  pub fn new(css: String, options: CompilerOptions) -> Self {
    Self {
      css,
      features: options.features,
      polyfills: options.polyfills,
      dependencies: options.dependencies,
      source_maps_enabled: options.source_maps_enabled,
      config: options.config,
    }
  }

  pub fn build(&mut self, _candidates: &[String]) -> String {
    self.css.clone()
  }

  pub fn build_source_map(&self) -> Option<String> {
    if !self.source_maps_enabled {
      return None;
    }

    Some(r#"{"version":3,"sources":[],"names":[],"mappings":""}"#.to_string())
  }

  pub fn dependencies(&self) -> &[String] {
    &self.dependencies
  }

  pub fn config(&self) -> Option<&TailwindConfig> {
    self.config.as_ref()
  }
}

pub fn compile(css: &str, options: CompilerOptions) -> Compiler {
  Compiler::new(css.to_string(), options)
}
