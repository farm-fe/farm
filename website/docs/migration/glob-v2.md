---
title: V2 glob migration
---

Farm V2 uses `farmfe_ecosystem_glob` throughout import glob matching, HMR matching and fixture traversal. The Rust Core re-export is now `farmfe_core::glob`; the former `farmfe_core::wax` export and its optional diagnostics are removed. This is an intentional breaking change before the stable V2 release.

## Rust consumers

Replace the old `wax::{Glob, Pattern}` import with `farmfe_core::glob::{Glob, WalkOptions}`. Matching returns `Result<bool, GlobError>`; handle an error separately from `Ok(false)`.

```rust
use farmfe_core::glob::{Glob, WalkOptions};

let glob = Glob::with_options("src/**/*.{js,ts}", WalkOptions::default())?;
for entry in glob.walk(root) {
  let entry = entry?;
  println!("{}", entry.path().display());
}
let matches = glob.is_match("src/main.ts")?;
```

Matching paths use forward slashes. The default walker excludes dot paths and `node_modules`; set `exhaustive: true` explicitly when needed. Farm's fixture helper uses exhaustive discovery to retain hidden and dependency-named fixture folders. Walk I/O and link-loop errors are propagated instead of silently dropping entries.

There is no `Pattern` trait requirement, `Glob::diagnosed`, `Glob::diagnose`, or default Wax `miette::Diagnostic` integration. Use ordinary `Display`, `Debug` and `std::error::Error`; the new error variants identify pattern compilation, walking, I/O and paths outside the root. There is no automatic fallback to Wax syntax.

## JavaScript consumers

The supported `import.meta.glob` adapter keeps its lazy/eager/import/query, alias/root/base, negative-pattern and HMR add/delete behavior. Query suffixes do not become object keys. The matcher is scoped to the fixed Vite 8.3.5 reference profiles, not every Wax, picomatch or minimatch dialect. Unclosed bracket handling follows the selected profile; direct matcher consumers can request strict brackets. Unsupported compilation and runtime matching errors remain errors, not empty match sets.

The transform/walker profile disables extglobs and enables POSIX classes. Direct HMR matching uses its distinct `posix: false` profile. Do not globally change one profile to imitate the other.

## Distribution and versions

The old Wax dependency is removed instead of merely disabling its diagnostics feature. Release qualification must verify the resolved all-target/default/all-feature graphs and newly rebuilt native payloads; earlier binaries cannot prove exclusion of the old dependency. This migration does not supply or invent missing upstream license materials.

Core/compiler/toolkit remain Rust V2. The plugin ABI stays `0.5.4` because Core data layouts and plugin interfaces do not change, but the release's Core and plugins must be rebuilt from the same new source cohort. The independently versioned glob crate remains `0.1.0`.

The accompanying Vuetify resolver fix is packaged as `@farmfe/js-plugin-vuetify@0.0.2`, not an overwrite of the published `0.0.1`. Packages and the official website are still subject to release approval and registry-only verification.
