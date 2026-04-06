# Farm Configuration Reference

Source docs: `website/docs/config/`

---

## Config File

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

- Use `farm start -c my-config.ts` to point to a different config file.
- **Do not use `__dirname`/`__filename`** — use `import.meta.url` instead (config is bundled to ESM).
- Alternatively use `farm.config.mjs` with `@type {import('@farmfe/core').UserConfig}` JSDoc.

---

## Shared (Root-Level) Options

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

## Compilation Options (`compilation.*`)

### Input / Output

```ts
compilation: {
  input: {
    index: './index.html',   // entry name → entry path
    about: './about.html',
  },
  output: {
    entryFilename: '[entryName].[ext]',    // default
    filename:      '[resourceName].[ext]', // default
    path:          'dist',                // output directory
    publicPath:    '/',                   // URL prefix for assets
    assetsFilename:'[resourceName].[ext]',
    targetEnv:     'browser-es2017',      // see targetEnv table below
    format:        'esm',                // see format below
    clean:         undefined,             // clean output.path before emit
    showFileSize:  true,
    asciiOnly:     false,
    externalGlobals: {},                 // { react: 'React' } for umd/iife
    name:          '__farm_global__',    // global name for umd/iife
  },
}
```

#### `output.targetEnv`

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

#### `output.format`

Supported: `esm` (default), `cjs`, `umd`, `iife`, `system`, `amd`.  
When `targetEnv: 'library'`, pass an **array**: `format: ['esm', 'cjs']` to produce multiple bundles in one build.

#### Filename Placeholders

`[entryName]`, `[resourceName]`, `[contentHash]`, `[ext]`

---

### Resolve

```ts
compilation: {
  resolve: {
    extensions:                ["tsx","mts","cts","ts","jsx","mjs","js","cjs","json","html","css"],
    alias:                     { "/@": "./src" },  // prefix replacement; $ for exact; $__farm_regex: for regex
    mainFields:                ["exports","browser","module","main"],
    conditions:                ["development","production","module"],
    symlinks:                  true,    // must be true when using pnpm
    strictExports:             false,
    autoExternalFailedResolve: false,
    dedupe:                    [],      // force single copy, e.g. ['react']
  },
}
```

Alias supports both object and array form; regex aliases use `$__farm_regex:^/(utils)$` prefix.

---

### Define

```ts
compilation: {
  define: {
    MY_VAR: 123,
    'process.env.API_URL': JSON.stringify('https://api.example.com'),
  },
}
```

Farm auto-injects `process.env.NODE_ENV` and internal HMR variables.

---

### External

```ts
compilation: {
  external: ['^stream$', { jquery: 'Jquery' }],
  externalNodeBuiltins: true,   // or an array of module names
}
```

---

### Script

```ts
compilation: {
  script: {
    target: 'esnext',    // es5 | es6 | es2015–es2023 | esnext
    parser: { /* SWC parser options — see https://swc.rs/docs/configuration/compilation#jscparser */ },
    plugins: [           // SWC plugins
      { name: 'swc-plugin-name', options: {}, filters: { moduleTypes: ['tsx'] } }
    ],
    decorators: {
      legacyDecorator:   true,
      decoratorMetadata: false,
      decoratorVersion:  '2021-12',  // or '2022-03'
      includes: [],
      excludes: ['node_modules/'],
    },
    nativeTopLevelAwait:   false,
    importNotUsedAsValues: 'remove', // 'remove' | 'preserve' | { preserve: string[] }
  },
}
```

---

### CSS

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
    transformToScript: true,  // true in dev (per-module HMR), false in prod
  },
}
```

---

### HTML

```ts
compilation: {
  html: {
    base: './base.html',  // All HTML entries inherit this base template
  },
}
```

---

### Source Maps

```ts
compilation: {
  sourcemap: true,  // true | false | 'inline' | 'all' | 'all-inline'
}
```

| Value | Behavior |
|-------|----------|
| `true` | Separate `.map` files, exclude node_modules (default) |
| `false` | Disabled |
| `'inline'` | Inline in output, exclude node_modules |
| `'all'` | Separate files for all modules incl. node_modules |
| `'all-inline'` | Inline for all modules |

---

### Partial Bundling

Farm's unique "partial bundling" balances bundle count vs size for optimal load performance.

```ts
compilation: {
  partialBundling: {
    targetConcurrentRequests: 25,      // target number of concurrent resource requests
    targetMinSize:            20480,   // 20 KB minimum resource size (before gzip)
    targetMaxSize:            1536000, // 1500 KB maximum
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

---

### Lazy Compilation

```ts
compilation: {
  lazyCompilation: true,  // default true in dev, false in build
}
```

---

### Tree Shaking

```ts
compilation: {
  treeShaking: true,  // default false in dev, true in build
}
```

---

### Minification

```ts
compilation: {
  minify: true,
  // or full config:
  minify: {
    compress: { /* TerserCompressOptions */ },
    mangle:   { /* TerserMangleOptions */ },
    include:  [],           // regex array — only in minify-module mode
    exclude:  ['*.min.js'],
    mode:     'minify-module', // 'minify-module' | 'minify-resource-pot'
  },
}
```

Default: `false` in dev, `true` in build.

---

### Preset Env (Polyfill)

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

---

### Persistent Cache

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

---

### Runtime

```ts
compilation: {
  runtime: {
    path:           undefined,              // custom runtime path (advanced)
    plugins:        [],                     // runtime plugin paths
    swcHelpersPath: undefined,
    namespace:      '<package.json name>',
    isolate:        false,                  // true = emit runtime as separate file
  },
}
```

---

### Assets

```ts
compilation: {
  assets: {
    include: ['txt'],  // additional file extensions to treat as static assets
  },
}
```

---

### Mode & Quick Disable

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

## Dev Server Options (`server.*`)

```ts
server: {
  port:           9000,
  host:           'localhost',
  strictPort:     false,
  open:           false,
  cors:           false,
  appType:        'spa',     // 'spa' | 'mpa' | 'custom'
  writeToDisk:    false,
  origin:         '',
  allowedHosts:   [],
  headers:        {},
  https:          undefined, // http2.SecureServerOptions
  hmr:            true,      // or HmrOptions object (see below)
  proxy:          {},        // http-proxy options keyed by path prefix
  middlewares:    [],
  middlewareMode: false,
  preview:        { /* preview server options */ },
}
```

### HMR Config

```ts
server: {
  hmr: {
    port:       undefined,  // WebSocket port (falls back to dev server port)
    host:       'localhost',
    clientPort: 9000,
    path:       '/__hmr',
    timeout:    0,
    overlay:    true,
    protocol:   '',         // 'ws' | 'wss' | '' (auto)
  },
}
```

### Proxy Config

```ts
server: {
  proxy: {
    '/api': {
      target:      'https://api.example.com',
      changeOrigin: true,
      pathRewrite: (path) => path.replace(/^\/api/, ''),
    },
  },
}
```

### Preview Server (`server.preview.*`)

| Option | Default | Description |
|--------|---------|-------------|
| `port` | `1911` | Preview server port |
| `host` | `'localhost'` | Host |
| `distDir` | `'dist'` | Built output dir |
| `open` | `false` | Auto-open browser |
| `strictPort` | `false` | Error on port conflict |
| `https` | inherits | HTTPS options |
| `headers` | inherits | Response headers |
| `proxy` | inherits | Proxy config |
| `middlewares` | `[]` | Custom middlewares |
