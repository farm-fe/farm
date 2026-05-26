# Design — `@farmfe/plugin-vue`

## Context

Two existing pieces of prior art shape this design:

1. **`unplugin-vue`** (TS, ~2.2k LoC) — the de-facto cross-bundler Vue SFC plugin. Its Farm entry point (`src/farm.ts`, 27 LoC) wraps the shared core (`src/core/index.ts`, 606 LoC) which registers `buildStart` / `resolveId` / `load` / `transform` and Farm-specific `config` / `configResolved` / `configureServer` / `updateModules` hooks. The shared core implements the `*.vue?vue&type=script|template|style&index=…&lang=…` virtual-module model.
2. **`fervid`** (Rust, v0.2) — exposes `fervid::compile(source, CompileOptions) -> CompileResult`. `CompileOptions` supports `filename`, `id`, `is_prod`, `is_custom_element`, `ssr`, `props_destructure`, `gen_default_as`, `source_map`, `transform_asset_urls`. `CompileResult` is one combined JS module plus `Vec<CompileEmittedStyle{code,is_compiled,lang,is_scoped}>` and `other_assets`. Fervid bakes script+template+setup together — it does **not** model `unplugin-vue`'s per-block split.
3. **`fervid_farmfe`** (in fervid's repo, ~145 LoC) — a proof-of-concept Farm plugin already wired via the `farm_plugin!` macro. Hooks: `load` (read `.vue` files as `ModuleType::Custom("vue")` + serve `?vue` style virtual modules) and `transform` (run `fervid::compile`, emit prepend-imports for style blocks). Missing: options surface, HMR, scoped-CSS preprocessing, custom-element filtering, source maps, devtools metadata, define-flag injection.

We build on top of the `fervid_farmfe` skeleton's shape, but rewrite it in-tree under `rust-plugins/vue/` to match Farm's plugin conventions and to layer on the rest of `unplugin-vue`'s observable surface.

## Strategic decisions

1. **Reuse fervid; do not re-implement or fork.** Re-writing fervid would be huge and out of scope. We depend on it as a published crate. Farm-side improvements that genuinely need fervid changes are filed upstream.
2. **Place the new plugin at `rust-plugins/vue/`** to live alongside `react`, `sass`, `dts`, `replace-dirname`. Crate `farmfe_plugin_vue`, npm package `@farmfe/plugin-vue`.
3. **Two-phase delivery.** Phase A ships the working subset that fervid supports today. Phase B layers on what `unplugin-vue` does that fervid does not (preprocessors, granular HMR, custom blocks, type-deps). Each phase is independently shippable.
4. **Defer non-fervid-able features to a thin JS companion later if needed.** Where fervid genuinely cannot help (CSS preprocessor invocation, `compiler-sfc`-style HMR descriptor diff that needs a cached prior descriptor), the Rust plugin emits hooks/events; a small JS wrapper drives the orchestration. Goal: keep the JS surface minimal — under ~100 LoC.
5. **Do not implement what fervid already implements differently.** E.g. fervid's scope-id is `compute_scope_id(source)`; `unplugin-vue` uses `hash(filepath)`. We accept fervid's algorithm in v0 and only surface `componentIdGenerator` if upstream lands it.

## Architecture

```
rust-plugins/vue/
├── Cargo.toml              # farmfe_plugin_vue crate, deps on farmfe_core + fervid
├── src/
│   ├── lib.rs              # FarmPluginVue: config, load, transform hooks
│   ├── options.rs          # serde-deserialised VuePluginOptions
│   ├── filter.rs           # include/exclude/customElement regex matching
│   ├── styles.rs           # style virtual module registry + lang→ModuleType
│   └── consts.rs           # query keys, default patterns
├── index.js                # platform binary resolver (copied from rust-plugins/react)
├── index.d.ts              # public Options TS type
├── func.js                 # default export wrapper → [binPath, options]
├── package.json            # @farmfe/plugin-vue, napi targets
├── npm/<targets>/          # per-arch shells
├── rustfmt.toml
└── tests/                  # hook-level integration tests
    ├── mod.rs
    └── fixtures/{basic,script-setup,scoped-style,css-vars,custom-element.ce,scss-style}.vue
```

### Hook responsibilities

- **`config` hook**: inject `__VUE_OPTIONS_API__` (default `true`), `__VUE_PROD_DEVTOOLS__` (default `false`), `__VUE_PROD_HYDRATION_MISMATCH_DETAILS__` (default `false`) into `compilation.define` only if not already user-set; add `'vue'` to `resolve.dedupe` when not targeting Node. Mirrors `unplugin-vue/src/core/index.ts` ~L380–410.

- **`load` hook**:
  - When `param.query` contains `vue=…` (any value) → look up the registered style virtual module by `param.module_id`, return `{content, module_type=Css|Custom(lang), source_map=None}`. Returns `None` when not registered, so other plugins can claim it.
  - When `param.resolved_path.ends_with(".vue")` and passes the `include` / `exclude` filter → read the file from disk, return content tagged with `ModuleType::Custom("vue")`.
  - Otherwise return `None`.

- **`transform` hook** (only acts when `module_type == ModuleType::Custom("vue")`):
  1. Decide `is_custom_element` = `param.resolved_path.ends_with(".ce.vue")` OR matches the configured `features.customElement` patterns.
  2. Call `fervid::compile(&param.content, CompileOptions { filename, id: module_id, is_prod, is_custom_element, ssr, props_destructure, source_map, … })`.
  3. For each emitted style block:
     - Build virtual id `<module_id>?vue&type=style&idx=<N>&lang=<lang>&scoped=<bool>`.
     - Register in the per-plugin `Mutex<FxHashMap<String, StyleEntry>>`.
     - Prepend `import "<resolved_path>?vue&type=style&…";` to the returned code.
  4. Append `__file = "<path>"` devtools metadata in development.
  5. Return `{ content, module_type: Some(ModuleType::Ts), source_map, ignore_previous_source_map: true }`.

### Style sub-block lang → `ModuleType` mapping

| `lang` returned by fervid | Plugin-emitted `ModuleType` | Claimed by |
|---|---|---|
| `""` / `"css"` | `ModuleType::Css` | Farm's built-in CSS pipeline |
| `"scss"` / `"sass"` | `ModuleType::Custom("scss"/"sass")` | `@farmfe/plugin-sass` |
| `"less"` / `"stylus"` | `ModuleType::Custom("less"/"stylus")` | Downstream Farm CSS preprocessor |

This is the key v0 lever for preprocessor support: we deliberately do **not** invoke sass/less inside the plugin. Farm's existing CSS preprocessor chain already handles them; we just tag the virtual module correctly.

### Options surface (Phase A)

Camel-cased, deserialised via serde from the JSON string Farm forwards to the constructor:

```ts
interface VuePluginOptions {
  include?: string | RegExp | (string | RegExp)[];   // default /\.vue$/
  exclude?: string | RegExp | (string | RegExp)[];
  isProduction?: boolean;     // default: derived from Farm's mode
  ssr?: boolean;              // default: false (experimental in fervid)
  sourceMap?: boolean;        // default: true
  customElement?: boolean | string | RegExp | (string | RegExp)[];  // deprecated alias
  features?: {
    optionsAPI?: boolean;                       // default: true
    prodDevtools?: boolean;                     // default: false
    prodHydrationMismatchDetails?: boolean;     // default: false
    propsDestructure?: boolean;                 // forwarded to fervid
    customElement?: boolean | string | RegExp | (string | RegExp)[];   // default /\.ce\.vue$/
  };
}
```

`unplugin-vue` options that are intentionally **absent** in Phase A:

- `compiler` — dropped; fervid is the compiler.
- `script` / `template` / `style` per-block config — fervid combines them in one call.
- `inlineTemplate: false` — fervid only emits an inline template.
- `features.componentIdGenerator` — fervid hard-codes `compute_scope_id(source)`.

## Fervid version pinning

We tried `fervid = "0.2.0"` from crates.io but it transitively depends on `swc_common 0.33.x`, which references `serde::__private` — that path has been removed from current `serde`, so the build fails.

Therefore we pin fervid to a known-good git rev (`phoenix-ru/fervid@4f26ab48`) until a fixed version publishes to crates.io. Once a published version restores buildability we swap back to a semver constraint. Tracked as a follow-up.

## CI / release wiring

Three workflows need plugin-vue added:

- `.github/workflows/rust-build.yaml` — extend the multi-target build step to invoke `npm run build` under `rust-plugins/vue/`, and add an `actions/upload-artifact` step for `rust-plugins/vue/npm/<abi>/index.farm`.
- `.github/workflows/ci.yaml` — extend the per-abi download/test matrix with `${{ github.sha }}-${{ matrix.settings.abi }}-plugin-vue` artifact downloads into `./rust-plugins/vue/npm/${{ matrix.settings.abi }}`; add `plugin-vue` to the rust-plugins compile matrix.
- `.github/workflows/release.yaml` — extend the artifact-move and existence-check loop to include plugin-vue alongside react/sass/replace-dirname.

## Testing strategy

Phase A ships with `rust-plugins/vue/tests/mod.rs`, a hook-level integration suite that exercises `load`, `transform`, and `config` directly against on-disk `.vue` fixtures. Snapshot testing of fervid's output is intentionally avoided in v0 because fervid's codegen is still under active development — pinning to byte-exact strings would cause spurious failures on every fervid upgrade. Instead each test asserts observable behaviour (which fixtures get compiled, which virtual modules get registered, which module types get tagged, which keys get inserted into `config.define`).

Future Phase B work should add E2E smoke tests by including a `examples/vue` reference project (deferred from this change to keep the diff reviewable).

## Risks and open questions

- **Fervid stability.** Upstream marks `compile()` as "Not production-ready yet." Phase A's tests exercise the API but not large real-world projects. We document the caveat in the docs page.
- **Crates.io publishability of fervid.** The git pin is a known workaround. If upstream takes a long time to ship a working `0.2.x`, we may need to vendor a thin wrapper.
- **Scope-id divergence.** Apps that rely on `unplugin-vue`'s `hash(filepath)` scope-id format may need adjustment. Phase B (or upstream fervid) can close this gap.
- **HMR coarseness in v0.** Phase A reloads the entire `.vue` module on any change; granular template-only / style-only HMR lands in Phase B and may require a JS companion.
