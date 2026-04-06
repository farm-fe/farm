---
name: farmfe-guide
description: >
  Comprehensive Farm build-tool reference for AI agents. Use when writing or reviewing farm.config.ts,
  adding/configuring plugins, troubleshooting builds, implementing SSR or library mode, using the
  JavaScript API, or writing Farm JS/Rust plugins. Covers all config options, official plugins,
  CLI commands, and key features sourced from website/docs.
license: MIT
compatibility: Farm ≥ 1.x. Node 16.18.0+. Uses pnpm as package manager.
metadata:
  author: farm
  version: "1.0"
references:
  - website/docs/config/configuring-farm.md
  - website/docs/config/compilation-options.md
  - website/docs/config/dev-server.md
  - website/docs/config/shared.md
  - website/docs/cli/cli-api.md
  - website/docs/using-plugins.mdx
  - website/docs/plugins/official-plugins/overview.md
  - website/docs/api/javascript-api.mdx
  - website/docs/api/js-plugin-api.md
  - website/docs/api/runtime-plugin-api.md
  - website/docs/features/css.md
  - website/docs/features/script.md
  - website/docs/features/library.md
  - website/docs/advanced/ssr.md
---

# Farm Build Tool — Complete Reference for AI Agents

Farm is an extremely fast Vite-compatible web build tool written in Rust.
Official site: https://farmfe.org  
Package: `@farmfe/core`  
Repo: `farm-fe/farm`

---

## 1. Quick Start

```bash
# Create new project
pnpm create farm@latest
pnpm create farm farm-app --template react   # specify template
```

Available templates: `vanilla`, `react`, `vue3`, `vue2`, `svelte`, `solid`, `preact`, `lit`, `nestjs`, `tauri`, `electron`.

```bash
# Install + start dev server
cd farm-app && pnpm install && pnpm dev     # http://localhost:9000

# Production build
pnpm build

# Preview production build
pnpm preview
```

Node requirement: **≥ 16.18.0**. Farm needs no config file for basic usage; defaults are sensible.

---

## 2. Config File

Farm reads `farm.config.ts|js|mjs` in the project root.

```ts title="farm.config.ts"
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  root: process.cwd(),        // project root, all relative paths resolved from here
  compilation: { /* ... */ }, // compiler options
  server:      { /* ... */ }, // dev-server options
  plugins:     [],            // Farm compilation plugins (Rust + JS)
  vitePlugins: [],            // Vite/Rollup/Unplugin plugins
});
```

Use `farm start -c my-config.ts` to point to a different config file.  
**Do not use `__dirname`/`__filename` in `farm.config.ts`** — use `import.meta.url` instead (config is bundled to ESM).  
Alternatively use `farm.config.mjs` with `@type {import('@farmfe/core').UserConfig}` JSDoc.

---

## 3. Shared (Root-Level) Options

| Option | Default | Description |
|--------|---------|-------------|
| `root` | `process.cwd()` | Project root. All relative paths resolve from here. |
| `clearScreen` | `true` | Clear terminal on recompile. |
| `mode` | `development` / `production` | Compilation mode. Overrides env. |
| `envDir` | `<root>` | Directory to load `.env` files. |
| `envPrefix` | `['FARM_', 'VITE_']` | Prefixes exposed to client via `import.meta.env`. |
| `publicDir` | `public` | Static assets dir — served as-is in dev, copied on build. |
| `watch` | `false` | File-watch config or boolean. |
| `plugins` | `[]` | Farm compilation plugins (Rust + JS). |
| `vitePlugins` | `[]` | Vite/Rollup/Unplugin plugins. |

---

## 4. Compilation Options (`compilation.*`)

### 4.1 Input / Output

```ts
compilation: {
  input: {
    index: './index.html',   // entry name → entry path
    about: './about.html',
  },
  output: {
    entryFilename: '[entryName].[ext]',   // default
    filename:      '[resourceName].[ext]',// default
    path:          'dist',               // output directory
    publicPath:    '/',                  // URL prefix for assets
    assetsFilename:'[resourceName].[ext]',
    targetEnv:     'browser-es2017',     // see §4.1.1
    format:        'esm',               // see §4.1.2
    clean:         undefined,            // clean output.path before emit
    showFileSize:  true,
    asciiOnly:     false,
    externalGlobals: {},                // { react: 'React' } for umd/iife
    name:          '__farm_global__',   // global name for umd/iife
  },
}
```

#### 4.1.1 `output.targetEnv`

| Value | Meaning |
|-------|---------|
| `browser` / `browser-es2017` | Default; browsers supporting async/await natively |
| `browser-es2015` | ES6 features |
| `browser-legacy` | ES5 / IE9 — large polyfill overhead |
| `browser-esnext` | Latest browsers, no polyfill |
| `node` / `node16` | Node 16 |
| `node-legacy` | Node 10 |
| `node-next` | Latest Node, no polyfill |
| `library` | Library mode (enables array `format`, bundle-less output) |

#### 4.1.2 `output.format`

Supported: `esm` (default), `cjs`, `umd`, `iife`, `system`, `amd`.  
When `targetEnv: 'library'`, pass an **array**: `format: ['esm', 'cjs']` to produce multiple bundles in one build.

#### 4.1.3 Filename Placeholders

`[entryName]`, `[resourceName]`, `[contentHash]`, `[ext]`

### 4.2 Resolve

```ts
compilation: {
  resolve: {
    extensions:               ["tsx","mts","cts","ts","jsx","mjs","js","cjs","json","html","css"],
    alias:                    { "/@": "./src" },   // prefix replacement; $ for exact match; $__farm_regex: for regex
    mainFields:               ["exports","browser","module","main"],
    conditions:               ["development","production","module"],
    symlinks:                 true,    // must be true when using pnpm
    strictExports:            false,
    autoExternalFailedResolve:false,
    dedupe:                   [],      // force single copy, e.g. ['react']
  },
}
```

Alias supports both object and array form; regex aliases use `$__farm_regex:^/(utils)$` prefix.

### 4.3 Define

```ts
compilation: {
  define: {
    MY_VAR: 123,
    'process.env.API_URL': JSON.stringify('https://api.example.com'),
  },
}
```

Farm auto-injects `process.env.NODE_ENV` and internal HMR variables.

### 4.4 External

```ts
compilation: {
  external: ['^stream$', { jquery: 'Jquery' }],
  externalNodeBuiltins: true,   // or an array of module names
}
```

### 4.5 Script

```ts
compilation: {
  script: {
    target: 'esnext',    // es5 | es6 | es2015–es2023 | esnext
    parser: { /* SWC parser options */ },
    plugins: [           // SWC plugins
      { name: 'swc-plugin-name', options: {}, filters: { moduleTypes: ['tsx'] } }
    ],
    decorators: {
      legacyDecorator:   true,
      decoratorMetadata: false,
      decoratorVersion:  '2021-12',   // or '2022-03'
      includes: [],
      excludes: ['node_modules/'],
    },
    nativeTopLevelAwait:     false,
    importNotUsedAsValues:   'remove', // 'remove' | 'preserve' | { preserve: string[] }
  },
}
```

### 4.6 CSS

```ts
compilation: {
  css: {
    modules: {
      paths:            ['\\.module\\.(css|scss|sass|less)'],
      identName:        '[name]-[hash]',
      localsConversion: 'asIs',  // 'asIs' | 'lowerCamel' | 'upperCamel' | 'snake'
    },
    prefixer: {
      targets: ['last 2 versions', '> 1%', 'ie >= 11'],
    },
    transformToScript: true,  // true in dev (HMR), false in prod
  },
}
```

### 4.7 HTML

```ts
compilation: {
  html: {
    base: './base.html',  // All HTML entries inherit this base template
  },
}
```

### 4.8 Source Maps

```ts
compilation: {
  sourcemap: true,        // true | false | 'inline' | 'all' | 'all-inline'
}
```

### 4.9 Partial Bundling

Farm's unique "partial bundling" balances bundle count vs size for optimal load performance.

```ts
compilation: {
  partialBundling: {
    targetConcurrentRequests: 25,       // target number of concurrent resource requests
    targetMinSize:            20480,    // 20KB minimum resource size (before gzip)
    targetMaxSize:            1536000,  // 1500KB maximum
    groups: [
      { name: 'vendor-react', test: ['node_modules/react'], groupType: 'immutable' }
    ],
    enforceResources: [
      { name: 'vendor', test: ['node_modules/'] }
    ],
    enforceTargetConcurrentRequests: false,
    enforceTargetMinSize:            false,
    immutableModules:                ['node_modules/'],
    immutableModulesWeight:          0.8,
  },
}
```

### 4.10 Lazy Compilation

```ts
compilation: {
  lazyCompilation: true,  // default true in dev, false in build
}
```

### 4.11 Tree Shaking

```ts
compilation: {
  treeShaking: true,  // default false in dev, true in build
}
```

### 4.12 Minification

```ts
compilation: {
  minify: true,
  // or full config:
  minify: {
    compress: { /* TerserCompressOptions */ },
    mangle:   { /* TerserMangleOptions */ },
    include:  [],           // regex array — only effective in minify-module mode
    exclude:  ['*.min.js'],
    mode:     'minify-module', // 'minify-module' | 'minify-resource-pot'
  },
}
```

Default: `false` in dev, `true` in build.

### 4.13 Preset Env (Polyfill)

```ts
compilation: {
  presetEnv: {
    include: ['node_modules/(es6-pkg|my-pkg)/'],
    exclude: ['node_modules/'],  // default
    options:     { /* swc preset-env options */ },
    assumptions: { /* swc assumptions */ },
  },
}
```

Default: `false` in dev, `true` in build. node_modules excluded by default.

### 4.14 Persistent Cache

```ts
compilation: {
  persistentCache: true,
  // or full config:
  persistentCache: {
    namespace:    'farm-cache',
    cacheDir:     'node_modules/.farm/cache',
    buildDependencies: ['path/to/plugin.js', 'farm-plugin-custom'],
    moduleCacheKeyStrategy: { timestamp: true, hash: true },
    envs: { MY_VAR: process.env.MY_VAR ?? '' },
    globalBuiltinCacheKeyStrategy: {
      define: true, buildDependencies: true, lockfile: true,
      packageJson: true, env: true,
    },
  },
}
```

Set `persistentCache: false` to disable. Cache stored in `node_modules/.farm/cache`.

### 4.15 Runtime

```ts
compilation: {
  runtime: {
    path:            undefined,          // custom runtime path (advanced)
    plugins:         [],                 // runtime plugin paths
    swcHelpersPath:  undefined,
    namespace:       '<package.json name>',
    isolate:         false,              // true = emit runtime as separate file
  },
}
```

### 4.16 Assets

```ts
compilation: {
  assets: {
    include: ['txt'],  // additional file extensions to treat as static assets
  },
}
```

### 4.17 Mode & Quick Disable

```ts
compilation: {
  mode: 'production',       // 'development' | 'production'
  lazyCompilation: false,
  persistentCache: false,
  minify:          false,
  treeShaking:     false,
}
```

---

## 5. Dev Server Options (`server.*`)

```ts
server: {
  port:        9000,
  host:        'localhost',
  strictPort:  false,
  open:        false,
  cors:        false,
  appType:     'spa',        // 'spa' | 'mpa' | 'custom'
  writeToDisk: false,
  origin:      '',
  allowedHosts: [],
  headers:     {},
  https:       undefined,    // http2.SecureServerOptions
  hmr:         true,         // or HmrOptions object
  proxy:       {},           // http-proxy options keyed by path prefix
  middlewares: [],
  middlewareMode: false,
  preview: { /* preview server options */ },
}
```

### 5.1 HMR Config

```ts
server: {
  hmr: {
    port:       undefined,    // WebSocket port (falls back to dev server port)
    host:       'localhost',
    clientPort: 9000,
    path:       '/__hmr',
    timeout:    0,
    overlay:    true,
    protocol:   '',           // 'ws' | 'wss' | '' (auto)
  },
}
```

### 5.2 Proxy Config

```ts
server: {
  proxy: {
    '/api': {
      target:     'https://api.example.com',
      changeOrigin: true,
      pathRewrite: (path) => path.replace(/^\/api/, ''),
    },
  },
}
```

### 5.3 Preview Server (`server.preview.*`)

| Option | Default | Description |
|--------|---------|-------------|
| `port` | `1911` | Preview server port |
| `host` | `localhost` | Host |
| `distDir` | `'dist'` | Built output dir |
| `open` | `false` | Auto-open browser |
| `strictPort` | `false` | Error on port conflict |
| `https` | inherits | HTTPS options |
| `headers` | inherits | Response headers |
| `proxy` | inherits | Proxy config |
| `middlewares` | `[]` | Custom middlewares |

---

## 6. CLI Commands

```bash
farm [root]                    # Start dev server
farm build                     # Production build
farm watch                     # Watch mode (equivalent to build --watch)
farm preview                   # Preview production build
farm clean [path]              # Clear persistent cache
farm plugin [command]          # Plugin management

# Common flags (available on most commands)
-c, --config <file>            # Use custom config file
-m, --mode <mode>              # Set env mode (development/production)
--port <port>
--host <host>
--open
--hmr
--strictPort
--base <path>                  # Public base path
-l, --lazy                     # Enable lazy compilation
--clearScreen

# Build-specific flags
-o, --outDir <dir>
-i, --input <file>
-w, --watch
--targetEnv <target>
--format <format>
--sourcemap
--treeShaking
--minify
```

---

## 7. Using Plugins

Farm supports four plugin types:

| Type | Config Key | Language | Notes |
|------|-----------|----------|-------|
| Farm Rust Plugin | `plugins` (string) | Rust | Fastest; use when available |
| Farm JS Plugin | `plugins` (object) | JS/TS | Rollup-style hooks |
| Vite/Rollup/Unplugin | `vitePlugins` | JS | Compatible out of box |
| SWC Plugin | `compilation.script.plugins` | Rust/WASM | Per-module transforms |
| Runtime Plugin | `compilation.runtime.plugins` | JS | Module system extensions |

### 7.1 Rust Plugins

```ts
plugins: [
  '@farmfe/plugin-sass',                        // string = Rust plugin package name
  ['@farmfe/plugin-sass', { additionalData: '' }], // [name, options] = with config
]
```

### 7.2 JS Plugins

```ts
import farmPostcss from '@farmfe/js-plugin-postcss';

plugins: [
  farmPostcss({ /* postcss options */ }),
  {
    name: 'my-plugin',
    priority: 100,     // default 100; Farm internal plugins are 99
    load: {
      filters: { resolvedPaths: ['\\.png$'] },
      executor: async ({ resolvedPath }) => ({
        content: `export default '${readFileSync(resolvedPath, 'base64')}'`,
        moduleType: 'js',
      }),
    },
  },
]
```

**`filters` are required** on every hook in JS plugins — unmatched modules are skipped entirely on the Rust side for performance.

### 7.3 Vite / Rollup / Unplugin

```ts
import vue from '@vitejs/plugin-vue';
import AutoImport from 'unplugin-auto-import/vite';

vitePlugins: [
  vue(),
  AutoImport({ /* options */ }),
  // Function form adds filters for performance:
  () => ({ vitePlugin: vue(), filters: ['\\.vue$'] }),
]
```

### 7.4 SWC Plugins

```ts
compilation: {
  script: {
    plugins: [{
      name: 'swc-plugin-vue-jsx',
      options: { transformOn: true },
      filters: { moduleTypes: ['tsx', 'jsx'] },
    }],
  },
}
```

### 7.5 Runtime Plugins

```ts
compilation: {
  runtime: {
    plugins: [
      require.resolve('farm-plugin-runtime-mock'),
      path.join(process.cwd(), 'build/runtime-plugin.ts'),
    ],
  },
}
```

---

## 8. Official Plugins Reference

### Rust Plugins

| Package | Description | Install |
|---------|-------------|---------|
| `@farmfe/plugin-react` | React JSX + react-refresh | `pnpm add -D @farmfe/plugin-react` |
| `@farmfe/plugin-sass` | Sass/SCSS compilation (sass-embedded) | `pnpm add -D @farmfe/plugin-sass` |
| `@farmfe/plugin-strip` | Remove debugger/console/assert statements | `pnpm add -D @farmfe/plugin-strip` |
| `@farmfe/plugin-dsv` | `.csv`/`.tsv` → JS modules | `pnpm add -D @farmfe/plugin-dsv` |
| `@farmfe/plugin-yaml` | YAML → ES modules | `pnpm add -D @farmfe/plugin-yaml` |
| `@farmfe/plugin-virtual` | Virtual modules | `pnpm add -D @farmfe/plugin-virtual` |
| `@farmfe/plugin-react-components` | Auto-import React components | `pnpm add -D @farmfe/plugin-react-components` |

### JS Plugins

| Package | Description | Install |
|---------|-------------|---------|
| `@farmfe/js-plugin-postcss` | PostCSS integration | `pnpm add -D @farmfe/js-plugin-postcss postcss` |
| `@farmfe/js-plugin-less` | Less compilation | `pnpm add -D @farmfe/js-plugin-less less` |
| `@farmfe/js-plugin-svgr` | SVG → React components | `pnpm add -D @farmfe/js-plugin-svgr` |
| `@farmfe/js-plugin-dts` | `.d.ts` generation | `pnpm add -D @farmfe/js-plugin-dts` |
| `@farmfe/js-plugin-sass` | Sass/SCSS (JS fallback) | `pnpm add -D @farmfe/js-plugin-sass sass` |
| `@farmfe/js-plugin-tailwindcss` | TailwindCSS integration | `pnpm add -D @farmfe/js-plugin-tailwindcss` |
| `@farmfe/js-plugin-visualizer` | Bundle size visualizer | `pnpm add -D @farmfe/js-plugin-visualizer` |
| `@farmfe/js-plugin-electron` | Electron app builds | `pnpm add -D @farmfe/js-plugin-electron` |

### Typical Plugin Configurations

**React:**
```ts
plugins: ['@farmfe/plugin-react']
```

**Vue (via Vite plugin):**
```ts
import vue from '@vitejs/plugin-vue';
vitePlugins: [vue()]
```

**Sass:**
```ts
plugins: [['@farmfe/plugin-sass', { additionalData: '@use "./global.scss";' }]]
```

**PostCSS + TailwindCSS:**
```ts
import postcss from '@farmfe/js-plugin-postcss';
plugins: [postcss()]
// postcss.config.js handles tailwind config
```

---

## 9. Key Features

### 9.1 CSS & Preprocessors

- CSS is supported out of the box.
- `.module.css|less|scss|sass` files are CSS Modules by default.
- Import CSS: `import './index.css'`
- Import CSS Module: `import styles from './index.module.css'`
- Sass: add `@farmfe/plugin-sass` Rust plugin.
- Less: add `@farmfe/js-plugin-less` JS plugin.
- PostCSS: add `@farmfe/js-plugin-postcss` JS plugin.

### 9.2 TypeScript / JSX / TSX

- Compiled with SWC out of the box.
- React JSX: use `@farmfe/plugin-react` (includes react-refresh for HMR).
- Decorators: configured via `compilation.script.decorators`.
- Target syntax: `compilation.script.target` (auto-set from `output.targetEnv`).

### 9.3 Environment Variables

- `.env`, `.env.local`, `.env.[mode]`, `.env.[mode].local` files are loaded.
- Variables prefixed with `FARM_` or `VITE_` are exposed to client code via `import.meta.env`.
- `import.meta.env.MODE`, `import.meta.env.DEV`, `import.meta.env.PROD` are always available.
- Custom prefix: set `envPrefix` at root level.
- Use `loadEnv(mode, dir, prefixes)` in config for programmatic access.

### 9.4 Static Assets

- Files in `publicDir` (default: `public/`) are served at root `/` in dev, copied to `output.path` in build.
- Import assets in JS: `import logo from './logo.png'` → URL string.
- Import raw: `import text from './file.txt?raw'`
- Import as URL: `import url from './img.png?url'`
- Add custom extensions to `compilation.assets.include`.

### 9.5 Source Maps

Set `compilation.sourcemap` to:
- `true` — separate `.map` files, exclude node_modules (default)
- `false` — disabled
- `'inline'` — inline in output, exclude node_modules
- `'all'` — separate files for all modules including node_modules
- `'all-inline'` — inline for all modules

### 9.6 Library Mode

```ts
compilation: {
  input: { index: './src/index.ts' },
  output: {
    targetEnv: 'library',
    format: ['esm', 'cjs'],          // array allowed in library mode
    libraryBundleType: 'bundle-less', // 'single-bundle' | 'multiple-bundle' | 'bundle-less'
  },
}
```

Bundle types:
- `single-bundle` — everything merged into one file per format (single entry only)
- `multiple-bundle` — each entry gets its own bundle, shared code split
- `bundle-less` — each source module becomes its own output file (tree-shakable for consumers)

### 9.7 Server-Side Rendering (SSR)

SSR requires two build configurations — one for the browser client, one for the Node server.

**Typical project structure:**
```
index.html               # Client HTML entry
farm.config.ts           # Browser build + dev server
farm.config.server.ts    # Node server build
server.js                # Production server script
src/
  index-client.tsx       # Client entry (hydration)
  index-server.tsx       # Server entry (renderToString)
  main.tsx               # Shared app code
```

**`farm.config.server.ts` example:**
```ts
export default defineConfig({
  compilation: {
    input:  { index: './src/index-server.tsx' },
    output: { targetEnv: 'node', format: 'cjs' },
  },
});
```

**Dev server middleware mode:**
```ts
// farm.config.ts
server: { middlewareMode: true }
```

SSR examples: `examples/react-ssr`, `examples/vue-ssr`, `examples/solid-ssr`.

### 9.8 Lazy Compilation

Enabled by default in dev mode (`compilation.lazyCompilation: true`). Routes/modules are compiled on first request, drastically speeding up cold start for large apps.

### 9.9 Persistent Cache

Enabled by default (`compilation.persistentCache: true`). Cache stored in `node_modules/.farm/cache`.  
Clear with: `farm clean` or set `persistentCache: false`.  
Cache is automatically invalidated when `farm.config.ts` or its dependencies change.

---

## 10. JavaScript API

```ts
import {
  start, build, watch, preview, clean,
  createCompiler, createDevServer, resolveConfig,
  loadEnv, Compiler, Server, logger,
} from '@farmfe/core';
```

### High-Level Functions

```ts
await start(inlineConfig);    // Start dev server
await build(inlineConfig);    // Production build
await watch(inlineConfig);    // Watch mode
await preview(inlineConfig);  // Preview built output
await clean(inlineConfig);    // Clear cache
```

### Lower-Level API

```ts
const config   = await resolveConfig(inlineConfig);
const compiler = await createCompiler(config);
const server   = await createDevServer(compiler, config);
server.listen();
```

### `Compiler` Methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `compile` | `() => Promise<void>` | Async compilation |
| `compileSync` | `() => void` | Sync compilation |
| `update` | `(paths: string[], sync?, ignoreCheck?) => JsUpdateResult` | Incremental update |
| `hasModule` | `(path: string) => boolean` | Check if module is in graph |
| `modules` | `() => Module[]` | All modules |
| `resources` | `() => Resource[]` | All output resources |
| `resource` | `(path: string) => Buffer \| null` | Single resource buffer |
| `writeResourcesToDisk` | `() => void` | Write output to disk |
| `outputPath` | `() => string` | Resolved output path |
| `traceDependencies` | `() => string[]` | All input dependencies |
| `resolvedModulePaths` | `(root: string) => string[]` | Module paths relative to root |
| `resolvedWatchPaths` | `() => string[]` | Watched paths |
| `getParentFiles` | `(resolvedPath: string) => string[]` | Parent importers |
| `addExtraWatchFile` | `(root, paths) => void` | Add watch files |
| `onUpdateFinish` | `(cb) => void` | Post-update callback |
| `removeOutputPathDir` | `() => void` | Delete output dir |

### `loadEnv`

```ts
const [env, files] = loadEnv(
  'development',              // mode
  '/path/to/project',         // envDir
  ['FARM_', 'VITE_']          // prefixes (default)
);
```

---

## 11. Writing JS Plugins

A Farm JS plugin is a plain object with hook properties.

```ts
import type { JsPlugin } from '@farmfe/core';

export default function myPlugin(options = {}): JsPlugin {
  return {
    name: 'my-plugin',
    priority: 100,         // higher = earlier execution; internal plugins = 99
    // hooks...
  };
}
```

### Available Hooks

| Hook | Type | When Called |
|------|------|-------------|
| `config` | serial | Modify config before compilation |
| `configResolved` | serial | After all config hooks |
| `configureDevServer` | serial | Dev server ready (dev only) |
| `configureCompiler` | serial | Rust compiler ready (dev + build) |
| `buildStart` | parallel | Before first compilation |
| `resolve` | first | Resolve module path |
| `load` | first | Load module content |
| `transform` | serial | Transform module content |
| `parse` | first | Parse AST |
| `renderStart` | parallel | Before resource generation |
| `renderResourcePot` | serial | Transform resource bundle |
| `augmentResourceHash` | serial | Add custom hash |
| `finalizeResources` | serial | After all resources generated |
| `writeResources` | serial | Write resources to disk |
| `pluginCacheLoaded` | serial | Cache loaded for plugin |
| `writePluginCache` | serial | Write plugin cache |
| `finish` | parallel | After compilation complete |
| `updateModules` | serial | HMR module update list |

### Hook Structure (JS Plugins)

All hooks that run per-module use `filters` for performance:

```ts
hookName: {
  filters: {
    resolvedPaths: ['\\.tsx?$'],   // regex strings
    moduleTypes:   ['js', 'ts'],   // module type names
  },
  async executor(params, context, hookContext) {
    // ...
    return result;
  },
},
```

### Common Hook Examples

**resolve:**
```ts
resolve: {
  filters: { sources: ['^my-virtual'], importers: ['.*'] },
  async executor({ source, importer, kind }, context) {
    if (source.startsWith('my-virtual:')) {
      return { resolvedPath: source, external: false, sideEffects: false, query: null, meta: null };
    }
  },
},
```

**load:**
```ts
load: {
  filters: { resolvedPaths: ['^virtual:'] },
  async executor({ resolvedPath }) {
    return { content: 'export const value = 42', moduleType: 'js', sourceMap: null };
  },
},
```

**transform:**
```ts
transform: {
  filters: { moduleTypes: ['js'] },
  async executor({ content, moduleId }) {
    return { content: content.replace('__VERSION__', '1.0.0'), sourceMap: null };
  },
},
```

**config:**
```ts
config(userConfig) {
  return { compilation: { resolve: { alias: { foo: 'bar' } } } };
},
```

---

## 12. Common Patterns & Recipes

### Monorepo / Workspace Alias

```ts
import path from 'path';
export default defineConfig({
  compilation: {
    resolve: {
      alias: { '@': path.resolve(process.cwd(), 'src') },
    },
  },
});
```

### CDN Public Path

```ts
output: {
  publicPath: process.env.NODE_ENV === 'production' ? 'https://cdn.example.com/' : '/',
}
```

### Vendor Bundling

```ts
compilation: {
  partialBundling: {
    groups: [{ name: 'vendor', test: ['node_modules/'] }],
  },
}
```

### Multiple HTML Entries (MPA)

```ts
compilation: {
  input: { index: './index.html', admin: './admin.html' },
},
server: { appType: 'mpa' },
```

### Disable All Optimizations (dev debugging)

```ts
compilation: {
  lazyCompilation: false,
  persistentCache: false,
  minify: false,
  treeShaking: false,
}
```

### HTTPS Dev Server

```ts
import fs from 'fs';
server: {
  https: {
    key:  fs.readFileSync('./localhost.key'),
    cert: fs.readFileSync('./localhost.crt'),
  },
}
```

### Custom Dev Middleware

```ts
server: {
  middlewares: [
    (server) => (req, res, next) => {
      if (req.url === '/health') { res.end('ok'); return; }
      next();
    },
  ],
}
```

---

## 13. Framework Quick Configs

### React

```ts
import { defineConfig } from '@farmfe/core';
export default defineConfig({
  plugins: ['@farmfe/plugin-react'],
});
```

### Vue 3

```ts
import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
export default defineConfig({
  vitePlugins: [vue()],
});
```

### Svelte

```ts
import { defineConfig } from '@farmfe/core';
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({
  vitePlugins: [svelte()],
});
```

### Electron

```ts
import { defineConfig } from '@farmfe/core';
import electron from '@farmfe/js-plugin-electron';
export default defineConfig({
  plugins: [electron({ /* options */ })],
});
```

---

## 14. Troubleshooting

| Symptom | Fix |
|---------|-----|
| Stale cache causing build errors | `farm clean` or set `persistentCache: false` |
| Port conflict | Set `server.strictPort: false` or change `server.port` |
| `__dirname` undefined in config | Use `import.meta.url` + `new URL('./...', import.meta.url).pathname` |
| Symlinked deps not resolved | Ensure `resolve.symlinks: true` (required for pnpm) |
| Type-only imports causing runtime errors | Set `script.importNotUsedAsValues: 'remove'` |
| Large bundle in legacy mode | `targetEnv: 'browser-legacy'` adds many polyfills; use `browser-es2017` unless IE needed |
| SWC plugin incompatibility | Pin to the swc_core version Farm uses; check plugin changelog |
| Vite plugin not working | Use `vitePlugins` array, not `plugins` |
| CSS not hot-reloading | Ensure `server.hmr: true`; CSS transformToScript helps in dev |

---

## 15. References

- **Docs**: https://farmfe.org/docs
- **Config Reference**: https://farmfe.org/docs/config/compilation-options
- **Plugin API**: https://farmfe.org/docs/api/js-plugin-api
- **JS API**: https://farmfe.org/docs/api/javascript-api
- **Awesome Farm**: https://github.com/farm-fe/awesome-farm
- **Examples**: `examples/` directory in this repo
- **Source docs**: `website/docs/` directory in this repo
