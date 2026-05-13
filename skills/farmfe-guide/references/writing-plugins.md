# Writing Custom Farm Plugins

Source docs: `website/docs/plugins/writing-plugins/`

---

## Plugin Types

Farm supports two kinds of custom plugins:

| Type | Language | Performance | When to use |
|------|----------|-------------|-------------|
| **JS Plugin** | TypeScript / JavaScript | Moderate | Quick integrations, wrapping existing tools |
| **Rust Plugin** | Rust | Fastest | Hot paths, AST-level transforms, native bindings |

Both are scaffolded with the same CLI and share the same hook pipeline.

---

## Scaffolding a New Plugin

Use `create-farm-plugin` to generate a ready-to-develop plugin project:

```bash
# Interactive wizard — choose JS or Rust
pnpm create farm-plugin

# Or create directly:
pnpm create farm-plugin my-farm-plugin --type js    # JS plugin
pnpm create farm-plugin my-farm-plugin --type rust  # Rust plugin
```

---

## Writing a JS Plugin

A JS plugin is a plain object (or factory function returning one) with hook properties.

### Minimal Example

```ts
// my-plugin.ts
import type { JsPlugin } from '@farmfe/core';

export interface MyPluginOptions {
  enable?: boolean;
}

export default function myPlugin(options: MyPluginOptions = {}): JsPlugin {
  return {
    name: 'my-plugin',  // required; must be non-empty
    priority: 100,      // optional; default 100. Higher = earlier. Internal plugins = 99.
    // hooks...
  };
}
```

Use in `farm.config.ts`:

```ts
import { defineConfig } from '@farmfe/core';
import myPlugin from './my-plugin';

export default defineConfig({
  plugins: [myPlugin({ enable: true })],
});
```

### Naming Conventions

- Prefix: `farm-plugin-` (e.g. `farm-plugin-my-feature`)
- Framework-specific prefixes: `farm-plugin-vue-*`, `farm-plugin-react-*`, `farm-plugin-svelte-*`
- Include `farm-plugin` keyword in `package.json`

### Key Concepts

- **`filters`** — required on per-module hooks (`resolve`, `load`, `transform`). JS plugins run on the JS side; unmatched modules are skipped entirely on the Rust side for performance.
- **`moduleType`** — identifies a module as `js`, `ts`, `css`, `html`, `json`, etc. Returned by `load` or `transform`.
- **`resolvedPath` vs `moduleId`** — `resolvedPath` is the absolute FS path; `moduleId` is `<project-relative-path>?<query>`.
- **`context`** — second argument to per-module hooks; gives access to the `ModuleGraph`, modules, and resources.

### Hook Skeleton

```ts
transform: {
  filters: {
    resolvedPaths: ['\\.md$'],  // regex strings matched against resolvedPath
    moduleTypes:   ['js'],      // module type names
  },
  async executor({ content, moduleId, moduleType }, context) {
    // mutate content, return updated result
    return {
      content:   content.replace('__VERSION__', '1.0.0'),
      sourceMap: null,
    };
  },
},
```

### Available JS Plugin Hooks

| Hook | Type | Phase | Description |
|------|------|-------|-------------|
| `config` | serial | startup | Modify `UserConfig` before compilation |
| `configResolved` | serial | startup | Read the final resolved config |
| `configureServer` | serial | startup | Configure the dev server (add middlewares) |
| `configureCompiler` | serial | startup | Access the compiler instance |
| `buildStart` | parallel | build | Before first compilation starts |
| `resolve` | first | build | Custom module resolution |
| `load` | first | build | Load module content |
| `transform` | serial | build | Transform module content |
| `processModule` | serial | build | Process a module after transformation |
| `freezeModule` | serial | build | Module finalized during build |
| `buildEnd` | parallel | build | After the build completes |
| `renderStart` | parallel | generate | Before resource generation |
| `processRenderedResourcePot` | serial | generate | Process a resource pot after rendering |
| `augmentResourcePotHash` | serial | generate | Contribute to a resource pot's hash |
| `finalizeResources` | serial | generate | Finalize resources before output |
| `transformHtml` | serial | generate | Transform generated HTML |
| `writeResources` | serial | generate | Called when resources are written |
| `pluginCacheLoaded` | serial | cache | Plugin cache loaded from disk |
| `writePluginCache` | serial | cache | Write plugin-specific cache |
| `finish` | parallel | done | After compilation is complete |
| `updateModules` | serial | HMR | Determine which modules to update |
| `updateFinished` | serial | HMR | HMR update complete |

**Hook execution types:**
- `first` — first plugin returning a non-null result wins.
- `serial` — all plugins run in priority order; each receives the previous result.
- `parallel` — all plugins run concurrently.

### Virtual Modules Pattern

```ts
export default function virtualPlugin(): JsPlugin {
  return {
    name: 'virtual-example',
    resolve: {
      filters: { sources: ['^virtual:example$'], importers: ['.*'] },
      async executor({ source }) {
        if (source === 'virtual:example') {
          return { resolvedPath: source, external: false, sideEffects: false, query: null, meta: null };
        }
      },
    },
    load: {
      filters: { resolvedPaths: ['^virtual:example$'] },
      async executor({ resolvedPath }) {
        return { content: 'export const msg = "hello from virtual";', moduleType: 'js' };
      },
    },
  };
}
```

---

## Writing a Rust Plugin

Rust plugins compile to a native `.farm` binary and are loaded dynamically at runtime.

### Minimal Example

```rust
#![deny(clippy::all)]

use farmfe_core::{config::Config, plugin::Plugin};
use farmfe_macro_plugin::farm_plugin;

#[farm_plugin]
pub struct FarmPluginExample {}

impl FarmPluginExample {
  // Must accept (&Config, String) — called by Farm on load
  fn new(_config: &Config, _options: String) -> Self {
    Self {}
  }
}

impl Plugin for FarmPluginExample {
  fn name(&self) -> &str {
    "FarmPluginExample"
  }

  // Override hooks as needed, e.g.:
  // fn load(...) -> ... { ... }
}
```

### Key Rules

- The struct must be `pub` and annotated with `#[farm_plugin]`.
- Must implement the `Plugin` trait with at minimum `name()`.
- `new(config: &Config, options: String)` is required for initialization.
- Use `farmfe_toolkit::lazy_static::lazy_static!{}` for static regex (not `once_cell`).
- Add `#![deny(clippy::all)]` at the top of `lib.rs`.
- `Cargo.toml` must declare `crate-type = ["cdylib", "rlib"]`.

### Cargo.toml (minimum)

```toml
[package]
name = "my_farm_plugin"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
farmfe_core = { workspace = true }
farmfe_macro_plugin = { workspace = true }
farmfe_toolkit = { workspace = true }
serde_json = "1"
```

### Project Structure

```
my-farm-plugin/
├── Cargo.toml
├── package.json
├── rust-toolchain.toml    # MUST match Farm core version — do not edit manually
├── src/
│   └── lib.rs
└── npm/
    ├── darwin-x64/
    ├── linux-x64-gnu/
    └── win32-x64-msvc/
```

### Build & Publish Scripts

```json
{
  "scripts": {
    "build": "farm-plugin-tools build --platform --cargo-name my_farm_plugin -p my_farm_plugin --release",
    "prepublishOnly": "farm-plugin-tools prepublish"
  }
}
```

### Available Rust Plugin Hooks

**Config:** `config`, `plugin_cache_loaded`

**Build:** `build_start`, `resolve`, `load`, `transform`, `parse`, `process_module`, `analyze_deps`, `finalize_module`, `freeze_module`, `module_graph_build_end`, `build_end`

**Generate:** `generate_start`, `optimize_module_graph`, `freeze_module_graph_meta`, `analyze_module_graph`, `partial_bundling`, `process_resource_pots`, `render_start`, `render_resource_pot`, `process_rendered_resource_pot`, `augment_resource_pot_hash`, `optimize_resource_pot`, `generate_resources`, `process_generated_resources`, `handle_entry_resource`, `finalize_resources`, `generate_end`

**Lifecycle:** `finish`, `write_plugin_cache`

**HMR:** `update_modules`, `module_graph_updated`, `update_finished`, `handle_persistent_cached_module`

> See [Rust Plugin API](https://farmfe.org/docs/api/rust-plugin-api) for full hook signatures.

---

## Publishing a Plugin

### JS Plugin

```bash
npm publish --access public
```

### Rust Plugin

Cross-build for all platforms using the provided GitHub Actions workflows, then:

```bash
# publish platform packages first
pnpm run prepublishOnly
# then publish the main package
npm publish --access public
```

> Platform packages under `npm/<platform>/` must be published before the root package.

---

## Tips

- Prefer **Rust plugins** for performance-critical paths (AST transforms, resolve, load).
- Prefer **JS plugins** for quick integrations or when wrapping an existing JS library.
- Use `priority > 99` to run before Farm's internal plugins; `< 99` to run after.
- Always define `filters` on per-module hooks to avoid unnecessary JS↔Rust round-trips.
- The `rust-toolchain.toml` in a Rust plugin **must** stay in sync with Farm core's toolchain version.
