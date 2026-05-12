use serde_json::Value;

use crate::ast::{self, AstNode};
use crate::design_system::DesignSystem;
use crate::parser::parse;
use crate::theme::Theme;

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
pub struct Compiler {
  design_system: DesignSystem,
  pub features: Features,
  pub polyfills: Polyfills,
  dependencies: Vec<String>,
  source_maps_enabled: bool,
  config: Option<TailwindConfig>,
}

impl Compiler {
  fn new(design_system: DesignSystem, features: Features, options: CompilerOptions) -> Self {
    Self {
      design_system,
      features,
      polyfills: options.polyfills,
      dependencies: options.dependencies,
      source_maps_enabled: options.source_maps_enabled,
      config: options.config,
    }
  }

  /// Build final CSS from the given candidate list.
  pub fn build(&mut self, candidates: &[String]) -> String {
    let nodes = self.design_system.compile_candidates(candidates);
    let optimized = ast::optimize_ast(nodes);
    ast::to_css(&optimized)
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

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum CompileError {
  ParseError(String),
}

impl std::fmt::Display for CompileError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      CompileError::ParseError(msg) => write!(f, "CSS parse error: {}", msg),
    }
  }
}

// ── compile() ─────────────────────────────────────────────────────────────────

/// Parse CSS, detect features, build a [`DesignSystem`], and return a
/// [`Compiler`].
pub fn compile(css: &str, options: CompilerOptions) -> Result<Compiler, CompileError> {
  let ast = parse(css);

  let features = detect_features(&ast);

  let theme = Theme::default();
  let design_system = DesignSystem::build(&ast, theme);

  Ok(Compiler::new(design_system, features, options))
}

// ── feature detection ─────────────────────────────────────────────────────────

/// Walk the AST and determine which Tailwind features are used.
fn detect_features(ast: &[AstNode]) -> Features {
  let mut features = Features::NONE;
  walk_features(ast, &mut features);
  features
}

fn walk_features(nodes: &[AstNode], features: &mut Features) {
  for node in nodes {
    match node {
      AstNode::AtRule(at_rule) => {
        match at_rule.name.as_str() {
          "@import" if at_rule.params.contains("tailwindcss") => {
            *features |= Features::UTILITIES;
          }
          "@tailwind" => {
            *features |= Features::UTILITIES;
          }
          "@apply" => {
            *features |= Features::AT_APPLY;
          }
          _ => {}
        }
        walk_features(&at_rule.nodes, features);
      }
      AstNode::Rule(rule) => {
        walk_features(&rule.nodes, features);
      }
      AstNode::Declaration(decl) => {
        if decl.property == "@apply" {
          *features |= Features::AT_APPLY;
        }
        if let Some(ref value) = decl.value {
          if value.contains("theme(") {
            *features |= Features::THEME_FUNCTION;
          }
        }
      }
      AstNode::Context(ctx) => walk_features(&ctx.nodes, features),
      AstNode::AtRoot(at_root) => walk_features(&at_root.nodes, features),
      _ => {}
    }
  }
}

