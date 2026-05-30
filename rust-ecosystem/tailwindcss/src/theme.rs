//! Tailwind CSS theme storage.
//!
//! Port of upstream `tailwindcss/src/theme.ts` (Tailwind v4). Stores theme
//! values keyed by their CSS-variable name (e.g. `--color-red-500`) along
//! with per-entry option flags. The same instance also holds collected
//! `@keyframes` definitions.

use crate::ast::AstNode;
use std::collections::HashMap;

/// Bit-flag option set for theme entries. Mirrors upstream `ThemeOptions`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ThemeOptions(pub u32);

impl ThemeOptions {
  pub const NONE: ThemeOptions = ThemeOptions(0);
  pub const INLINE: ThemeOptions = ThemeOptions(1 << 0);
  pub const REFERENCE: ThemeOptions = ThemeOptions(1 << 1);
  pub const DEFAULT: ThemeOptions = ThemeOptions(1 << 2);
  pub const STATIC: ThemeOptions = ThemeOptions(1 << 3);
  pub const USED: ThemeOptions = ThemeOptions(1 << 4);

  pub fn contains(self, other: ThemeOptions) -> bool {
    (self.0 & other.0) == other.0
  }

  pub fn intersects(self, other: ThemeOptions) -> bool {
    (self.0 & other.0) != 0
  }
}

impl std::ops::BitOr for ThemeOptions {
  type Output = Self;
  fn bitor(self, rhs: Self) -> Self {
    ThemeOptions(self.0 | rhs.0)
  }
}

impl std::ops::BitOrAssign for ThemeOptions {
  fn bitor_assign(&mut self, rhs: Self) {
    self.0 |= rhs.0;
  }
}

impl std::ops::BitAnd for ThemeOptions {
  type Output = Self;
  fn bitand(self, rhs: Self) -> Self {
    ThemeOptions(self.0 & rhs.0)
  }
}

/// A single resolved value stored in the [`Theme`].
#[derive(Debug, Clone)]
pub struct ThemeValue {
  pub value: String,
  pub options: ThemeOptions,
}

/// Keys in `ignoredThemeKeyMap` from upstream `theme.ts`. When the user queries
/// the `--font` namespace, keys like `--font-weight` and `--font-size`
/// (which are independent namespaces despite the shared prefix) are excluded.
fn ignored_theme_keys_for(namespace: &str) -> &'static [&'static str] {
  match namespace {
    "--font" => &["--font-weight", "--font-size"],
    "--inset" => &["--inset-shadow", "--inset-ring"],
    "--text" => &[
      "--text-color",
      "--text-decoration-color",
      "--text-decoration-thickness",
      "--text-indent",
      "--text-shadow",
      "--text-underline-offset",
    ],
    "--grid-column" => &["--grid-column-start", "--grid-column-end"],
    "--grid-row" => &["--grid-row-start", "--grid-row-end"],
    _ => &[],
  }
}

fn is_ignored_theme_key(theme_key: &str, namespace: &str) -> bool {
  ignored_theme_keys_for(namespace)
    .iter()
    .any(|ignored| theme_key == *ignored || theme_key.starts_with(&format!("{ignored}-")))
}

/// Resolved Tailwind theme.
#[derive(Debug, Clone, Default)]
pub struct Theme {
  /// Optional prefix (e.g. `tw`) applied to every variable when emitted.
  pub prefix: Option<String>,
  values: HashMap<String, ThemeValue>,
  keyframes: Vec<AstNode>,
}

impl Theme {
  /// Create an empty theme.
  pub fn new() -> Self {
    Self::default()
  }

  /// Create a theme seeded with Tailwind v4's default theme tokens. This is a
  /// minimal but representative slice intended to make built-in functional
  /// utilities (spacing, colors, radius, font-size, etc.) produce sensible
  /// output without an external configuration. Mirrors the most commonly used
  /// values from upstream `tailwindcss/preflight.css` / `theme.css`.
  pub fn with_defaults() -> Self {
    let mut t = Self::new();
    let d = ThemeOptions::DEFAULT;

    // ── Spacing base ────────────────────────────────────────────────────
    t.add("--spacing", "0.25rem", d);

    // ── Color palette (slate / gray / red / orange / yellow / green / blue / indigo / purple / pink / black / white / transparent / current) ──
    let palette: &[(&str, &str)] = &[
      ("--color-black", "#000"),
      ("--color-white", "#fff"),
      ("--color-transparent", "transparent"),
      ("--color-current", "currentColor"),
      // Slate
      ("--color-slate-50", "oklch(98.4% 0.003 247.858)"),
      ("--color-slate-100", "oklch(96.8% 0.007 247.896)"),
      ("--color-slate-200", "oklch(92.9% 0.013 255.508)"),
      ("--color-slate-300", "oklch(86.9% 0.022 252.894)"),
      ("--color-slate-400", "oklch(70.4% 0.04 256.788)"),
      ("--color-slate-500", "oklch(55.4% 0.046 257.417)"),
      ("--color-slate-600", "oklch(44.6% 0.043 257.281)"),
      ("--color-slate-700", "oklch(37.2% 0.044 257.287)"),
      ("--color-slate-800", "oklch(27.9% 0.041 260.031)"),
      ("--color-slate-900", "oklch(20.8% 0.042 265.755)"),
      // Gray
      ("--color-gray-50", "oklch(98.5% 0.002 247.839)"),
      ("--color-gray-100", "oklch(96.7% 0.003 264.542)"),
      ("--color-gray-200", "oklch(92.8% 0.006 264.531)"),
      ("--color-gray-300", "oklch(87.2% 0.01 258.338)"),
      ("--color-gray-400", "oklch(70.7% 0.022 261.325)"),
      ("--color-gray-500", "oklch(55.1% 0.027 264.364)"),
      ("--color-gray-600", "oklch(44.6% 0.03 256.802)"),
      ("--color-gray-700", "oklch(37.3% 0.034 259.733)"),
      ("--color-gray-800", "oklch(27.8% 0.033 256.848)"),
      ("--color-gray-900", "oklch(21% 0.034 264.665)"),
      // Red
      ("--color-red-50", "oklch(97.1% 0.013 17.38)"),
      ("--color-red-100", "oklch(93.6% 0.032 17.717)"),
      ("--color-red-200", "oklch(88.5% 0.062 18.334)"),
      ("--color-red-300", "oklch(80.8% 0.114 19.571)"),
      ("--color-red-400", "oklch(70.4% 0.191 22.216)"),
      ("--color-red-500", "oklch(63.7% 0.237 25.331)"),
      ("--color-red-600", "oklch(57.7% 0.245 27.325)"),
      ("--color-red-700", "oklch(50.5% 0.213 27.518)"),
      ("--color-red-800", "oklch(44.4% 0.177 26.899)"),
      ("--color-red-900", "oklch(39.6% 0.141 25.723)"),
      // Blue
      ("--color-blue-50", "oklch(97% 0.014 254.604)"),
      ("--color-blue-100", "oklch(93.2% 0.032 255.585)"),
      ("--color-blue-200", "oklch(88.2% 0.059 254.128)"),
      ("--color-blue-300", "oklch(80.9% 0.105 251.813)"),
      ("--color-blue-400", "oklch(70.7% 0.165 254.624)"),
      ("--color-blue-500", "oklch(62.3% 0.214 259.815)"),
      ("--color-blue-600", "oklch(54.6% 0.245 262.881)"),
      ("--color-blue-700", "oklch(48.8% 0.243 264.376)"),
      ("--color-blue-800", "oklch(42.4% 0.199 265.638)"),
      ("--color-blue-900", "oklch(37.9% 0.146 265.522)"),
      // Green
      ("--color-green-50", "oklch(98.2% 0.018 155.826)"),
      ("--color-green-100", "oklch(96.2% 0.044 156.743)"),
      ("--color-green-200", "oklch(92.5% 0.084 155.995)"),
      ("--color-green-300", "oklch(87.1% 0.15 154.449)"),
      ("--color-green-400", "oklch(79.2% 0.209 151.711)"),
      ("--color-green-500", "oklch(72.3% 0.219 149.579)"),
      ("--color-green-600", "oklch(62.7% 0.194 149.214)"),
      ("--color-green-700", "oklch(52.7% 0.154 150.069)"),
      ("--color-green-800", "oklch(44.8% 0.119 151.328)"),
      ("--color-green-900", "oklch(39.3% 0.095 152.535)"),
      // Yellow
      ("--color-yellow-50", "oklch(98.7% 0.026 102.212)"),
      ("--color-yellow-100", "oklch(97.3% 0.071 103.193)"),
      ("--color-yellow-200", "oklch(94.5% 0.129 101.54)"),
      ("--color-yellow-300", "oklch(90.5% 0.182 98.111)"),
      ("--color-yellow-400", "oklch(85.2% 0.199 91.936)"),
      ("--color-yellow-500", "oklch(79.5% 0.184 86.047)"),
      ("--color-yellow-600", "oklch(68.1% 0.162 75.834)"),
      ("--color-yellow-700", "oklch(55.4% 0.135 66.442)"),
      ("--color-yellow-800", "oklch(47.6% 0.114 61.907)"),
      ("--color-yellow-900", "oklch(42.1% 0.095 57.708)"),
      // Indigo
      ("--color-indigo-50", "oklch(96.2% 0.018 272.314)"),
      ("--color-indigo-100", "oklch(93% 0.034 272.788)"),
      ("--color-indigo-200", "oklch(87% 0.065 274.039)"),
      ("--color-indigo-300", "oklch(78.5% 0.115 274.713)"),
      ("--color-indigo-400", "oklch(67.3% 0.182 276.935)"),
      ("--color-indigo-500", "oklch(58.5% 0.233 277.117)"),
      ("--color-indigo-600", "oklch(51.1% 0.262 276.966)"),
      ("--color-indigo-700", "oklch(45.7% 0.24 277.023)"),
      ("--color-indigo-800", "oklch(39.8% 0.195 277.366)"),
      ("--color-indigo-900", "oklch(35.9% 0.144 278.697)"),
    ];
    for (k, v) in palette {
      t.add(k, v, d);
    }

    // ── Border radius ───────────────────────────────────────────────────
    t.add("--radius-xs", "0.125rem", d);
    t.add("--radius-sm", "0.25rem", d);
    t.add("--radius-md", "0.375rem", d);
    t.add("--radius-lg", "0.5rem", d);
    t.add("--radius-xl", "0.75rem", d);
    t.add("--radius-2xl", "1rem", d);
    t.add("--radius-3xl", "1.5rem", d);
    t.add("--radius-4xl", "2rem", d);

    // ── Font size with nested line-height ───────────────────────────────
    let text: &[(&str, &str, &str)] = &[
      ("--text-xs", "0.75rem", "calc(1 / 0.75)"),
      ("--text-sm", "0.875rem", "calc(1.25 / 0.875)"),
      ("--text-base", "1rem", "calc(1.5 / 1)"),
      ("--text-lg", "1.125rem", "calc(1.75 / 1.125)"),
      ("--text-xl", "1.25rem", "calc(1.75 / 1.25)"),
      ("--text-2xl", "1.5rem", "calc(2 / 1.5)"),
      ("--text-3xl", "1.875rem", "calc(2.25 / 1.875)"),
      ("--text-4xl", "2.25rem", "calc(2.5 / 2.25)"),
      ("--text-5xl", "3rem", "1"),
      ("--text-6xl", "3.75rem", "1"),
      ("--text-7xl", "4.5rem", "1"),
      ("--text-8xl", "6rem", "1"),
      ("--text-9xl", "8rem", "1"),
    ];
    for (k, size, lh) in text {
      t.add(k, size, d);
      t.add(&format!("{k}--line-height"), lh, d);
    }

    // ── Font weight ─────────────────────────────────────────────────────
    let weight: &[(&str, &str)] = &[
      ("--font-weight-thin", "100"),
      ("--font-weight-extralight", "200"),
      ("--font-weight-light", "300"),
      ("--font-weight-normal", "400"),
      ("--font-weight-medium", "500"),
      ("--font-weight-semibold", "600"),
      ("--font-weight-bold", "700"),
      ("--font-weight-extrabold", "800"),
      ("--font-weight-black", "900"),
    ];
    for (k, v) in weight {
      t.add(k, v, d);
    }

    // ── Aspect ratio ────────────────────────────────────────────────────
    t.add("--aspect-video", "16 / 9", d);

    t
  }

  /// Total number of theme entries.
  pub fn size(&self) -> usize {
    self.values.len()
  }

  /// Mirrors upstream `Theme.add(key, value, options)`.
  ///
  /// Special cases:
  /// - `--ns-*: initial` clears every key under namespace `--ns-`.
  /// - `--*: initial` clears all theme values.
  /// - `value == "initial"` deletes the entry.
  /// - When `DEFAULT` is requested and a non-default entry already exists, the
  ///   new value is ignored (existing user value wins).
  pub fn add(&mut self, key: &str, value: &str, options: ThemeOptions) {
    if let Some(stripped) = key.strip_suffix("-*") {
      if value != "initial" {
        // Upstream throws; we just no-op (parser ensures `initial` upstream).
        return;
      }
      if key == "--*" {
        self.values.clear();
      } else {
        self.clear_namespace(stripped, ThemeOptions::NONE);
      }
      return;
    }

    if options.contains(ThemeOptions::DEFAULT) {
      if let Some(existing) = self.values.get(key) {
        if !existing.options.contains(ThemeOptions::DEFAULT) {
          return;
        }
      }
    }

    if value == "initial" {
      self.values.remove(key);
    } else {
      self.values.insert(
        key.to_string(),
        ThemeValue {
          value: value.to_string(),
          options,
        },
      );
    }
  }

  /// Insert a raw entry (low-level escape hatch, mostly for tests).
  pub fn insert(&mut self, key: &str, value: &str) {
    self.add(key, value, ThemeOptions::NONE);
  }

  /// Mirrors upstream `Theme.keysInNamespaces`. Returns the *suffixes*
  /// (without the leading `<namespace>-`) of every entry that belongs to one
  /// of the given namespaces.
  pub fn keys_in_namespaces(&self, theme_keys: &[&str]) -> Vec<String> {
    let mut keys = Vec::new();

    for namespace in theme_keys {
      let prefix = format!("{namespace}-");

      for key in self.values.keys() {
        if !key.starts_with(&prefix) {
          continue;
        }
        // Skip nested sub-variables (contain a second `--`).
        if key[2..].contains("--") {
          continue;
        }
        if is_ignored_theme_key(key, namespace) {
          continue;
        }
        keys.push(key[prefix.len()..].to_string());
      }
    }

    keys
  }

  /// Return the first matching theme value by raw theme key (no candidate
  /// suffix substitution). Mirrors upstream `Theme.get`.
  pub fn get(&self, theme_keys: &[&str]) -> Option<&str> {
    for key in theme_keys {
      if let Some(v) = self.values.get(*key) {
        return Some(&v.value);
      }
    }
    None
  }

  pub fn has_default(&self, key: &str) -> bool {
    self.get_options(key).contains(ThemeOptions::DEFAULT)
  }

  pub fn get_options(&self, key: &str) -> ThemeOptions {
    let unprefixed = self.unprefix_key(key);
    self
      .values
      .get(&unprefixed)
      .map(|v| v.options)
      .unwrap_or(ThemeOptions::NONE)
  }

  /// Iterate all `(key, value)` entries in insertion order is not guaranteed
  /// (HashMap); use [`Theme::entries_sorted`] for deterministic output.
  pub fn entries(&self) -> impl Iterator<Item = (String, &str)> + '_ {
    self
      .values
      .iter()
      .map(move |(k, v)| (self.prefix_key(k), v.value.as_str()))
  }

  /// Sorted version of [`Theme::entries`], for deterministic output.
  pub fn entries_sorted(&self) -> Vec<(String, String)> {
    let mut out: Vec<_> = self
      .values
      .iter()
      .map(|(k, v)| (self.prefix_key(k), v.value.clone()))
      .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
  }

  /// Apply the configured prefix to a key. With `prefix = "tw"`,
  /// `--color-red-500` → `--tw-color-red-500`.
  pub fn prefix_key(&self, key: &str) -> String {
    match &self.prefix {
      None => key.to_string(),
      Some(p) => format!("--{p}-{}", &key[2..]),
    }
  }

  fn unprefix_key(&self, key: &str) -> String {
    match &self.prefix {
      None => key.to_string(),
      Some(p) => {
        // strip `--<prefix>-` (= 3 + prefix.len() chars).
        let drop = 3 + p.len();
        if key.len() > drop {
          format!("--{}", &key[drop..])
        } else {
          key.to_string()
        }
      }
    }
  }

  /// Remove every theme entry whose key starts with `namespace`. When
  /// `clear_options` is non-zero, only entries whose options contain every
  /// flag in `clear_options` are removed.
  pub fn clear_namespace(&mut self, namespace: &str, clear_options: ThemeOptions) {
    let ignored = ignored_theme_keys_for(namespace);
    let to_remove: Vec<String> = self
      .values
      .iter()
      .filter(|(key, value)| {
        if !key.starts_with(namespace) {
          return false;
        }
        if clear_options != ThemeOptions::NONE && !value.options.contains(clear_options) {
          return false;
        }
        !ignored.iter().any(|ig| key.starts_with(*ig))
      })
      .map(|(k, _)| k.clone())
      .collect();
    for k in to_remove {
      self.values.remove(&k);
    }
  }

  fn resolve_key(&self, candidate_value: Option<&str>, theme_keys: &[&str]) -> Option<String> {
    for namespace in theme_keys {
      let mut theme_key = match candidate_value {
        Some(v) => format!("{namespace}-{v}"),
        None => namespace.to_string(),
      };

      if !self.values.contains_key(&theme_key) {
        // If a candidate value contains dots, Tailwind also tries replacing
        // them with underscores (e.g. `1.5` → `1_5`).
        if let Some(v) = candidate_value {
          if v.contains('.') {
            theme_key = format!("{namespace}-{}", v.replace('.', "_"));
            if !self.values.contains_key(&theme_key) {
              continue;
            }
          } else {
            continue;
          }
        } else {
          continue;
        }
      }

      if is_ignored_theme_key(&theme_key, namespace) {
        continue;
      }

      return Some(theme_key);
    }
    None
  }

  fn var_expr(&self, theme_key: &str) -> Option<String> {
    let value = self.values.get(theme_key)?;
    let mut fallback = String::new();
    if value.options.contains(ThemeOptions::REFERENCE) {
      fallback.push_str(", ");
      fallback.push_str(&value.value);
    }
    Some(format!("var({}{})", self.prefix_key(theme_key), fallback))
  }

  /// Mirrors upstream `Theme.resolve`. Returns either the inline value (when
  /// the entry or `options` are `INLINE`) or a `var(--...)` reference.
  pub fn resolve(
    &self,
    candidate_value: Option<&str>,
    theme_keys: &[&str],
    options: ThemeOptions,
  ) -> Option<String> {
    let theme_key = self.resolve_key(candidate_value, theme_keys)?;
    let value = self.values.get(&theme_key)?;
    if (options | value.options).contains(ThemeOptions::INLINE) {
      return Some(value.value.clone());
    }
    self.var_expr(&theme_key)
  }

  /// Like [`Theme::resolve`] but always returns the raw value, never a
  /// `var(...)` reference.
  pub fn resolve_value(
    &self,
    candidate_value: Option<&str>,
    theme_keys: &[&str],
  ) -> Option<String> {
    let theme_key = self.resolve_key(candidate_value, theme_keys)?;
    self.values.get(&theme_key).map(|v| v.value.clone())
  }

  /// Mirrors upstream `Theme.resolveWith`. Resolves the candidate plus a set
  /// of nested keys (e.g. `--text-sm--line-height`). Returns the main value
  /// and a map of nested-key → resolved-value.
  pub fn resolve_with(
    &self,
    candidate_value: &str,
    theme_keys: &[&str],
    nested_keys: &[&str],
  ) -> Option<(String, HashMap<String, String>)> {
    let theme_key = self.resolve_key(Some(candidate_value), theme_keys)?;
    let mut extra = HashMap::new();
    for name in nested_keys {
      let nested_key = format!("{theme_key}{name}");
      let Some(nested_value) = self.values.get(&nested_key) else {
        continue;
      };
      let v = if nested_value.options.contains(ThemeOptions::INLINE) {
        nested_value.value.clone()
      } else {
        self.var_expr(&nested_key)?
      };
      extra.insert(name.to_string(), v);
    }
    let value = self.values.get(&theme_key)?;
    let main = if value.options.contains(ThemeOptions::INLINE) {
      value.value.clone()
    } else {
      self.var_expr(&theme_key)?
    };
    Some((main, extra))
  }

  /// Return all entries that belong to a namespace, keyed by the trailing
  /// suffix (`None` when the entry equals the namespace exactly).
  pub fn namespace(&self, namespace: &str) -> HashMap<Option<String>, String> {
    let mut values = HashMap::new();
    let prefix = format!("{namespace}-");
    let nested_prefix = format!("{namespace}--");

    for (key, value) in &self.values {
      if key == namespace {
        values.insert(None, value.value.clone());
      } else if key.starts_with(&nested_prefix) {
        // Preserve `--` prefix for sub-variables, e.g. `--font-size-sm--line-height`.
        values.insert(
          Some(key[namespace.len()..].to_string()),
          value.value.clone(),
        );
      } else if key.starts_with(&prefix) {
        values.insert(Some(key[prefix.len()..].to_string()), value.value.clone());
      }
    }
    values
  }

  pub fn add_keyframes(&mut self, value: AstNode) {
    self.keyframes.push(value);
  }

  pub fn get_keyframes(&self) -> &[AstNode] {
    &self.keyframes
  }

  /// Marks a theme variable as used. Returns `true` if this was the first time
  /// the variable was marked used (used by upstream to decide whether to emit
  /// the `@property` fallback).
  pub fn mark_used_variable(&mut self, theme_key: &str) -> bool {
    let key = self.unprefix_key(theme_key);
    let Some(value) = self.values.get_mut(&key) else {
      return false;
    };
    let was_used = value.options.contains(ThemeOptions::USED);
    value.options |= ThemeOptions::USED;
    !was_used
  }

  // -- Back-compatibility helpers ------------------------------------------

  /// Look up a CSS variable directly by its full name (e.g. `--color-red-500`).
  /// Kept for callers that already hold the literal variable name and only
  /// need its value.
  pub fn lookup_var(&self, name: &str) -> Option<String> {
    self.values.get(name).map(|v| v.value.clone())
  }

  /// Resolve a dot-path theme key like `colors.red.500` to its raw value.
  ///
  /// Conversion rules (kept identical to the pre-Phase-14 implementation so
  /// that `theme(colors.red.500)` substitution in [`crate::functions`] stays
  /// stable):
  /// - `colors.red.500` → `--color-red-500` (trailing `s` on the first
  ///   segment is stripped).
  /// - `spacing.4` → `--spacing-4` (no trailing `s`).
  pub fn resolve_by_key_path(&self, key_path: &str) -> Option<String> {
    let var_name = key_path_to_var(key_path);
    self.lookup_var(&var_name)
  }
}

fn key_path_to_var(key_path: &str) -> String {
  let parts: Vec<&str> = key_path.split('.').collect();
  let mut result = String::from("--");
  for (i, part) in parts.iter().enumerate() {
    if i > 0 {
      result.push('-');
    }
    if i == 0 && part.ends_with('s') && part.len() > 1 {
      result.push_str(&part[..part.len() - 1]);
    } else {
      result.push_str(part);
    }
  }
  result
}
