# Farm JS Plugin API Reference

Source docs: `website/docs/api/js-plugin-api.md`, `website/docs/plugins/writing-plugins/js-plugin.mdx`

---

## Plugin Shape

A Farm JS plugin is a plain object with hook properties, exported from a factory function:

```ts
import type { JsPlugin } from '@farmfe/core';

export interface MyPluginOptions {
  enable?: boolean;
}

export default function myPlugin(options: MyPluginOptions = {}): JsPlugin {
  return {
    name: 'my-plugin',      // required; must be non-empty
    priority: 100,          // optional; default 100. Higher = earlier. Internal plugins = 99.
    // hooks...
  };
}
```

> Use `create-farm-plugin` CLI to scaffold a new JS plugin with the correct structure.

---

## Hook Reference

| Hook | Hook Type | Phase | When Called |
|------|-----------|-------|-------------|
| `config` | serial | startup | Modify `UserConfig` before compilation |
| `configResolved` | serial | startup | After all `config` hooks; receive final config |
| `configureDevServer` | serial | startup | Dev server ready (dev only) |
| `configureCompiler` | serial | startup | Rust compiler ready (dev + build) |
| `buildStart` | parallel | build | Before first compilation starts |
| `resolve` | first | build | Resolve a module source to a path |
| `load` | first | build | Load module content from a resolved path |
| `transform` | serial | build | Transform module content (after load) |
| `parse` | first | build | Parse module AST |
| `renderStart` | parallel | generate | Before resource generation |
| `renderResourcePot` | serial | generate | Transform a resource bundle |
| `augmentResourceHash` | serial | generate | Add custom hash contribution |
| `finalizeResources` | serial | generate | After all resources are generated |
| `writeResources` | serial | generate | Write resources to disk |
| `pluginCacheLoaded` | serial | cache | Plugin cache has been loaded |
| `writePluginCache` | serial | cache | Write plugin-specific cache data |
| `finish` | parallel | done | After compilation completes |
| `updateModules` | serial | HMR | Receive list of updated modules for HMR |

**Hook types:**
- `first` — first plugin to return a non-null result wins; rest are skipped.
- `serial` — all plugins run in priority order; each receives the previous result.
- `parallel` — all plugins run concurrently.

---

## Per-Module Hook Structure

Per-module hooks (`resolve`, `load`, `transform`, `parse`) use `filters` to avoid executing on unrelated modules. **Filters are required** — unmatched modules are skipped entirely on the Rust side.

```ts
hookName: {
  filters: {
    resolvedPaths: ['\\.tsx?$'],  // regex strings matched against the resolved path
    moduleTypes:   ['js', 'ts'],  // module type names
  },
  async executor(params, context, hookContext) {
    // return result or null/undefined to pass to next plugin
  },
},
```

---

## Hook Examples

### `config` — Modify Config

```ts
config(userConfig) {
  return {
    compilation: {
      resolve: { alias: { foo: 'bar' } },
    },
  };
},
```

`config` is called after user plugins are resolved; adding new plugins in `config` has no effect.

---

### `configResolved` — Read Final Config

```ts
const myPlugin = () => {
  let farmConfig;
  return {
    name: 'my-plugin',
    configResolved(resolved) {
      farmConfig = resolved;
    },
  };
};
```

---

### `configureDevServer` — Access Dev Server

```ts
configureDevServer(server) {
  // server is the Farm Server instance
  console.log('Dev server ready:', server);
},
```

---

### `resolve` — Custom Module Resolution

```ts
resolve: {
  filters: {
    sources:   ['^my-virtual:'],
    importers: ['.*'],
  },
  async executor({ source, importer, kind }, context, hookContext) {
    if (source.startsWith('my-virtual:')) {
      return {
        resolvedPath: source,
        external:     false,
        sideEffects:  false,
        query:        null,
        meta:         null,
      };
    }
  },
},
```

To call `context.resolve` recursively, pass `caller` to avoid infinite loops:

```ts
async executor(param, context, hookContext) {
  if (hookContext.caller === 'my-plugin') return null;
  const newSource = param.source.replace('foo', 'bar');
  return context.resolve({ ...param, source: newSource }, { caller: 'my-plugin', meta: {} });
},
```

> By default `resolve` runs **after** Farm's internal resolver. To override default resolution, set `priority > 101`.

---

### `load` — Load Module Content

```ts
load: {
  filters: { resolvedPaths: ['^virtual:my-plugin$'] },
  async executor({ resolvedPath, query, meta }) {
    return {
      content:   'export const value = 42;',
      moduleType: 'js',
      sourceMap:  null,
    };
  },
},
```

---

### `transform` — Transform Module Content

```ts
transform: {
  filters: { moduleTypes: ['js'] },
  async executor({ content, moduleId, moduleType }) {
    return {
      content:   content.replace('__VERSION__', '1.0.0'),
      sourceMap: null,
    };
  },
},
```

---

### `buildStart` — Initialization

```ts
buildStart: {
  async executor() {
    // runs once before first compilation; not called for HMR updates
    await myPlugin.initialize();
  },
},
```

---

### `finish` — Post-Compilation

```ts
finish: {
  async executor() {
    console.log('Build complete!');
  },
},
```

---

## `CompilationContext` (second arg)

Available in `resolve`, `load`, `transform`, `parse` hooks.

```ts
interface CompilationContext {
  resolve(
    param: PluginResolveHookParam,
    hookContext: { caller?: string; meta: Record<string, unknown> }
  ): Promise<PluginResolveHookResult | null>;
  // ... other utilities
}
```

---

## Key Types

```ts
interface PluginResolveHookParam {
  source:   string;        // import specifier, e.g. './App.vue'
  importer: string | null; // absolute path of the importing module
  kind:     ResolveKind;   // 'import' | 'require' | 'cssImport' | ...
}

interface PluginResolveHookResult {
  resolvedPath: string;
  external:     boolean;
  sideEffects:  boolean;
  query:        [string, string][] | null;
  meta:         Record<string, string> | null;
}

interface PluginLoadHookResult {
  content:    string;
  moduleType: ModuleType;  // 'js' | 'ts' | 'css' | 'html' | ...
  sourceMap?: string | null;
}
```
