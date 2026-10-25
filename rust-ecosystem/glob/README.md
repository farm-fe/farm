# Farm scoped glob compatibility

This crate derives its pure Rust matcher from **picomatch-rs 0.1.0**, commit
`ceb39c90fc719fbbab79216a139f31c174272ed3`, published archive SHA256
`76a98ffe0fdca4e658785b8f9a5557c71815e57bb4cf3ce36d8a470b1046329a`.
It does not use the upstream NAPI wrapper or JavaScript RegExp.

Reference profiles are Vite **8.3.5**'s JS transform/tinyglobby 0.2.17
(picomatch 4.0.7, fdir 6.5.0) and direct JS HMR matching. This is a scoped
compatibility implementation, **not a claim of full picomatch/minimatch/Vite
compatibility**, including Vite's separate native Rolldown path.

Farm changes preserve ordinary parenthesized-group dot behavior, respect strict
unclosed brackets and pattern length limits, and avoid Rust POSIX-class expansion
when that option is disabled. `UPSTREAM.patch` records the functional matcher
changes against the published source; the derived Rust files are then formatted
with this repository's `rustfmt.toml`. The patch applies before that formatting.
`WalkOptions` uses the transform profile: extglobs disabled, POSIX classes enabled,
dot files and node_modules excluded by default; exhaustive enables both. Traversal
uses walkdir with symlink loop/I/O errors propagated, not converted to no matches.
Matching paths use forward slashes. Results are sorted/deduplicated by the Farm
adapter. Unsupported/invalid regex compilation returns an error without fallback.

The direct HMR profile must set `posix:false`; it must not inherit walker defaults.
The Farm adapter still owns aliases, root/base paths, output keys, import queries,
AST validation and change invalidation. Core now re-exports this crate through
`farmfe_core::glob`, and fixture traversal uses it too. The former Wax export and
optional diagnostic API are removed before stable V2. Dependency exclusion and
freshly rebuilt package payloads must both be verified before qualifying the new
distribution; this does not supply missing original upstream license materials.

Original source: https://github.com/Maidang1/picomatch-rs/tree/ceb39c90fc719fbbab79216a139f31c174272ed3/crates/picomatch-rs
Original project license: https://github.com/Maidang1/picomatch-rs/blob/ceb39c90fc719fbbab79216a139f31c174272ed3/LICENSE
Preserve `UPSTREAM-LICENSE` verbatim (including Jon Schlinkert's copyright) and
Farm's `LICENSE`. No new copyright holder or grant has been invented.
