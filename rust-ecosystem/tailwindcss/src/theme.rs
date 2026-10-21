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

    // Default color tokens from Tailwind CSS 4.1.12 theme.css (MIT, Tailwind Labs).
    // Keep every family and shade so standard color utilities resolve consistently.
    let palette: &[(&str, &str)] = &[
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
      ("--color-red-950", "oklch(25.8% 0.092 26.042)"),
      ("--color-orange-50", "oklch(98% 0.016 73.684)"),
      ("--color-orange-100", "oklch(95.4% 0.038 75.164)"),
      ("--color-orange-200", "oklch(90.1% 0.076 70.697)"),
      ("--color-orange-300", "oklch(83.7% 0.128 66.29)"),
      ("--color-orange-400", "oklch(75% 0.183 55.934)"),
      ("--color-orange-500", "oklch(70.5% 0.213 47.604)"),
      ("--color-orange-600", "oklch(64.6% 0.222 41.116)"),
      ("--color-orange-700", "oklch(55.3% 0.195 38.402)"),
      ("--color-orange-800", "oklch(47% 0.157 37.304)"),
      ("--color-orange-900", "oklch(40.8% 0.123 38.172)"),
      ("--color-orange-950", "oklch(26.6% 0.079 36.259)"),
      ("--color-amber-50", "oklch(98.7% 0.022 95.277)"),
      ("--color-amber-100", "oklch(96.2% 0.059 95.617)"),
      ("--color-amber-200", "oklch(92.4% 0.12 95.746)"),
      ("--color-amber-300", "oklch(87.9% 0.169 91.605)"),
      ("--color-amber-400", "oklch(82.8% 0.189 84.429)"),
      ("--color-amber-500", "oklch(76.9% 0.188 70.08)"),
      ("--color-amber-600", "oklch(66.6% 0.179 58.318)"),
      ("--color-amber-700", "oklch(55.5% 0.163 48.998)"),
      ("--color-amber-800", "oklch(47.3% 0.137 46.201)"),
      ("--color-amber-900", "oklch(41.4% 0.112 45.904)"),
      ("--color-amber-950", "oklch(27.9% 0.077 45.635)"),
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
      ("--color-yellow-950", "oklch(28.6% 0.066 53.813)"),
      ("--color-lime-50", "oklch(98.6% 0.031 120.757)"),
      ("--color-lime-100", "oklch(96.7% 0.067 122.328)"),
      ("--color-lime-200", "oklch(93.8% 0.127 124.321)"),
      ("--color-lime-300", "oklch(89.7% 0.196 126.665)"),
      ("--color-lime-400", "oklch(84.1% 0.238 128.85)"),
      ("--color-lime-500", "oklch(76.8% 0.233 130.85)"),
      ("--color-lime-600", "oklch(64.8% 0.2 131.684)"),
      ("--color-lime-700", "oklch(53.2% 0.157 131.589)"),
      ("--color-lime-800", "oklch(45.3% 0.124 130.933)"),
      ("--color-lime-900", "oklch(40.5% 0.101 131.063)"),
      ("--color-lime-950", "oklch(27.4% 0.072 132.109)"),
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
      ("--color-green-950", "oklch(26.6% 0.065 152.934)"),
      ("--color-emerald-50", "oklch(97.9% 0.021 166.113)"),
      ("--color-emerald-100", "oklch(95% 0.052 163.051)"),
      ("--color-emerald-200", "oklch(90.5% 0.093 164.15)"),
      ("--color-emerald-300", "oklch(84.5% 0.143 164.978)"),
      ("--color-emerald-400", "oklch(76.5% 0.177 163.223)"),
      ("--color-emerald-500", "oklch(69.6% 0.17 162.48)"),
      ("--color-emerald-600", "oklch(59.6% 0.145 163.225)"),
      ("--color-emerald-700", "oklch(50.8% 0.118 165.612)"),
      ("--color-emerald-800", "oklch(43.2% 0.095 166.913)"),
      ("--color-emerald-900", "oklch(37.8% 0.077 168.94)"),
      ("--color-emerald-950", "oklch(26.2% 0.051 172.552)"),
      ("--color-teal-50", "oklch(98.4% 0.014 180.72)"),
      ("--color-teal-100", "oklch(95.3% 0.051 180.801)"),
      ("--color-teal-200", "oklch(91% 0.096 180.426)"),
      ("--color-teal-300", "oklch(85.5% 0.138 181.071)"),
      ("--color-teal-400", "oklch(77.7% 0.152 181.912)"),
      ("--color-teal-500", "oklch(70.4% 0.14 182.503)"),
      ("--color-teal-600", "oklch(60% 0.118 184.704)"),
      ("--color-teal-700", "oklch(51.1% 0.096 186.391)"),
      ("--color-teal-800", "oklch(43.7% 0.078 188.216)"),
      ("--color-teal-900", "oklch(38.6% 0.063 188.416)"),
      ("--color-teal-950", "oklch(27.7% 0.046 192.524)"),
      ("--color-cyan-50", "oklch(98.4% 0.019 200.873)"),
      ("--color-cyan-100", "oklch(95.6% 0.045 203.388)"),
      ("--color-cyan-200", "oklch(91.7% 0.08 205.041)"),
      ("--color-cyan-300", "oklch(86.5% 0.127 207.078)"),
      ("--color-cyan-400", "oklch(78.9% 0.154 211.53)"),
      ("--color-cyan-500", "oklch(71.5% 0.143 215.221)"),
      ("--color-cyan-600", "oklch(60.9% 0.126 221.723)"),
      ("--color-cyan-700", "oklch(52% 0.105 223.128)"),
      ("--color-cyan-800", "oklch(45% 0.085 224.283)"),
      ("--color-cyan-900", "oklch(39.8% 0.07 227.392)"),
      ("--color-cyan-950", "oklch(30.2% 0.056 229.695)"),
      ("--color-sky-50", "oklch(97.7% 0.013 236.62)"),
      ("--color-sky-100", "oklch(95.1% 0.026 236.824)"),
      ("--color-sky-200", "oklch(90.1% 0.058 230.902)"),
      ("--color-sky-300", "oklch(82.8% 0.111 230.318)"),
      ("--color-sky-400", "oklch(74.6% 0.16 232.661)"),
      ("--color-sky-500", "oklch(68.5% 0.169 237.323)"),
      ("--color-sky-600", "oklch(58.8% 0.158 241.966)"),
      ("--color-sky-700", "oklch(50% 0.134 242.749)"),
      ("--color-sky-800", "oklch(44.3% 0.11 240.79)"),
      ("--color-sky-900", "oklch(39.1% 0.09 240.876)"),
      ("--color-sky-950", "oklch(29.3% 0.066 243.157)"),
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
      ("--color-blue-950", "oklch(28.2% 0.091 267.935)"),
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
      ("--color-indigo-950", "oklch(25.7% 0.09 281.288)"),
      ("--color-violet-50", "oklch(96.9% 0.016 293.756)"),
      ("--color-violet-100", "oklch(94.3% 0.029 294.588)"),
      ("--color-violet-200", "oklch(89.4% 0.057 293.283)"),
      ("--color-violet-300", "oklch(81.1% 0.111 293.571)"),
      ("--color-violet-400", "oklch(70.2% 0.183 293.541)"),
      ("--color-violet-500", "oklch(60.6% 0.25 292.717)"),
      ("--color-violet-600", "oklch(54.1% 0.281 293.009)"),
      ("--color-violet-700", "oklch(49.1% 0.27 292.581)"),
      ("--color-violet-800", "oklch(43.2% 0.232 292.759)"),
      ("--color-violet-900", "oklch(38% 0.189 293.745)"),
      ("--color-violet-950", "oklch(28.3% 0.141 291.089)"),
      ("--color-purple-50", "oklch(97.7% 0.014 308.299)"),
      ("--color-purple-100", "oklch(94.6% 0.033 307.174)"),
      ("--color-purple-200", "oklch(90.2% 0.063 306.703)"),
      ("--color-purple-300", "oklch(82.7% 0.119 306.383)"),
      ("--color-purple-400", "oklch(71.4% 0.203 305.504)"),
      ("--color-purple-500", "oklch(62.7% 0.265 303.9)"),
      ("--color-purple-600", "oklch(55.8% 0.288 302.321)"),
      ("--color-purple-700", "oklch(49.6% 0.265 301.924)"),
      ("--color-purple-800", "oklch(43.8% 0.218 303.724)"),
      ("--color-purple-900", "oklch(38.1% 0.176 304.987)"),
      ("--color-purple-950", "oklch(29.1% 0.149 302.717)"),
      ("--color-fuchsia-50", "oklch(97.7% 0.017 320.058)"),
      ("--color-fuchsia-100", "oklch(95.2% 0.037 318.852)"),
      ("--color-fuchsia-200", "oklch(90.3% 0.076 319.62)"),
      ("--color-fuchsia-300", "oklch(83.3% 0.145 321.434)"),
      ("--color-fuchsia-400", "oklch(74% 0.238 322.16)"),
      ("--color-fuchsia-500", "oklch(66.7% 0.295 322.15)"),
      ("--color-fuchsia-600", "oklch(59.1% 0.293 322.896)"),
      ("--color-fuchsia-700", "oklch(51.8% 0.253 323.949)"),
      ("--color-fuchsia-800", "oklch(45.2% 0.211 324.591)"),
      ("--color-fuchsia-900", "oklch(40.1% 0.17 325.612)"),
      ("--color-fuchsia-950", "oklch(29.3% 0.136 325.661)"),
      ("--color-pink-50", "oklch(97.1% 0.014 343.198)"),
      ("--color-pink-100", "oklch(94.8% 0.028 342.258)"),
      ("--color-pink-200", "oklch(89.9% 0.061 343.231)"),
      ("--color-pink-300", "oklch(82.3% 0.12 346.018)"),
      ("--color-pink-400", "oklch(71.8% 0.202 349.761)"),
      ("--color-pink-500", "oklch(65.6% 0.241 354.308)"),
      ("--color-pink-600", "oklch(59.2% 0.249 0.584)"),
      ("--color-pink-700", "oklch(52.5% 0.223 3.958)"),
      ("--color-pink-800", "oklch(45.9% 0.187 3.815)"),
      ("--color-pink-900", "oklch(40.8% 0.153 2.432)"),
      ("--color-pink-950", "oklch(28.4% 0.109 3.907)"),
      ("--color-rose-50", "oklch(96.9% 0.015 12.422)"),
      ("--color-rose-100", "oklch(94.1% 0.03 12.58)"),
      ("--color-rose-200", "oklch(89.2% 0.058 10.001)"),
      ("--color-rose-300", "oklch(81% 0.117 11.638)"),
      ("--color-rose-400", "oklch(71.2% 0.194 13.428)"),
      ("--color-rose-500", "oklch(64.5% 0.246 16.439)"),
      ("--color-rose-600", "oklch(58.6% 0.253 17.585)"),
      ("--color-rose-700", "oklch(51.4% 0.222 16.935)"),
      ("--color-rose-800", "oklch(45.5% 0.188 13.697)"),
      ("--color-rose-900", "oklch(41% 0.159 10.272)"),
      ("--color-rose-950", "oklch(27.1% 0.105 12.094)"),
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
      ("--color-slate-950", "oklch(12.9% 0.042 264.695)"),
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
      ("--color-gray-950", "oklch(13% 0.028 261.692)"),
      ("--color-zinc-50", "oklch(98.5% 0 0)"),
      ("--color-zinc-100", "oklch(96.7% 0.001 286.375)"),
      ("--color-zinc-200", "oklch(92% 0.004 286.32)"),
      ("--color-zinc-300", "oklch(87.1% 0.006 286.286)"),
      ("--color-zinc-400", "oklch(70.5% 0.015 286.067)"),
      ("--color-zinc-500", "oklch(55.2% 0.016 285.938)"),
      ("--color-zinc-600", "oklch(44.2% 0.017 285.786)"),
      ("--color-zinc-700", "oklch(37% 0.013 285.805)"),
      ("--color-zinc-800", "oklch(27.4% 0.006 286.033)"),
      ("--color-zinc-900", "oklch(21% 0.006 285.885)"),
      ("--color-zinc-950", "oklch(14.1% 0.005 285.823)"),
      ("--color-neutral-50", "oklch(98.5% 0 0)"),
      ("--color-neutral-100", "oklch(97% 0 0)"),
      ("--color-neutral-200", "oklch(92.2% 0 0)"),
      ("--color-neutral-300", "oklch(87% 0 0)"),
      ("--color-neutral-400", "oklch(70.8% 0 0)"),
      ("--color-neutral-500", "oklch(55.6% 0 0)"),
      ("--color-neutral-600", "oklch(43.9% 0 0)"),
      ("--color-neutral-700", "oklch(37.1% 0 0)"),
      ("--color-neutral-800", "oklch(26.9% 0 0)"),
      ("--color-neutral-900", "oklch(20.5% 0 0)"),
      ("--color-neutral-950", "oklch(14.5% 0 0)"),
      ("--color-stone-50", "oklch(98.5% 0.001 106.423)"),
      ("--color-stone-100", "oklch(97% 0.001 106.424)"),
      ("--color-stone-200", "oklch(92.3% 0.003 48.717)"),
      ("--color-stone-300", "oklch(86.9% 0.005 56.366)"),
      ("--color-stone-400", "oklch(70.9% 0.01 56.259)"),
      ("--color-stone-500", "oklch(55.3% 0.013 58.071)"),
      ("--color-stone-600", "oklch(44.4% 0.011 73.639)"),
      ("--color-stone-700", "oklch(37.4% 0.01 67.558)"),
      ("--color-stone-800", "oklch(26.8% 0.007 34.298)"),
      ("--color-stone-900", "oklch(21.6% 0.006 56.043)"),
      ("--color-stone-950", "oklch(14.7% 0.004 49.25)"),
      ("--color-black", "#000"),
      ("--color-white", "#fff"),
      ("--color-transparent", "transparent"),
      ("--color-current", "currentColor"),
    ];
    for (key, value) in palette {
      t.add(key, value, d);
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
