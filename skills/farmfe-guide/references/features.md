# Farm Features Reference

Source docs: `website/docs/features/`, `website/docs/advanced/`, `website/docs/quick-start.mdx`

---

## Quick Start

```bash
# Scaffold a new project
pnpm create farm@latest
pnpm create farm farm-app --template react   # specify template directly

# Available templates
# vanilla, react, vue3, vue2, svelte, solid, preact, lit, nestjs, tauri, electron

# Install + run
cd farm-app && pnpm install && pnpm dev      # http://localhost:9000
pnpm build                                   # production build → dist/
pnpm preview                                 # preview production build
```

Node requirement: **≥ 16.18.0**.

---

## CSS & Preprocessors

- CSS is supported out of the box — just `import './index.css'`.
- HMR for CSS is automatic in dev mode.
- `.module.css|less|scss|sass` → CSS Modules by default.

```tsx
// plain CSS
import './index.css';

// CSS Module
import styles from './index.module.css';
export function Comp() { return <div className={styles.main} />; }
```

**CSS Modules configuration:**

```ts
compilation: {
  css: {
    modules: {
      paths: ['\\.module\\.(css|scss|sass|less)'],
      identName: '[name]-[hash]',
      localsConversion: 'asIs',  // 'asIs' | 'lowerCamel' | 'upperCamel' | 'snake'
    },
  },
}
```

**Preprocessors (install the plugin, then use normally):**

| Preprocessor | Plugin | Install |
|-------------|--------|---------|
| Sass / SCSS | `@farmfe/plugin-sass` (Rust) | `pnpm add -D @farmfe/plugin-sass` |
| Less | `@farmfe/js-plugin-less` (JS) | `pnpm add -D @farmfe/js-plugin-less less` |
| PostCSS | `@farmfe/js-plugin-postcss` (JS) | `pnpm add -D @farmfe/js-plugin-postcss postcss` |

---

## TypeScript / JSX / TSX

- Compiled with **SWC** out of the box — no additional config needed.
- React JSX: add `@farmfe/plugin-react` (includes react-refresh for HMR).
- Vue JSX: use `@vitejs/plugin-vue` + `@vitejs/plugin-vue-jsx` in `vitePlugins`.
- Decorators: configured via `compilation.script.decorators`.
- Target syntax version: `compilation.script.target` (auto-set from `output.targetEnv`).

```ts title="farm.config.ts"
// Enabling decorators example
compilation: {
  script: {
    decorators: { legacyDecorator: true },
  },
}
```

---

## Environment Variables

- Farm loads `.env`, `.env.local`, `.env.[mode]`, `.env.[mode].local`.
- Variables with `FARM_` or `VITE_` prefix are exposed to client code via `import.meta.env`.
- Built-in: `import.meta.env.MODE`, `import.meta.env.DEV`, `import.meta.env.PROD`.
- Custom prefix: set `envPrefix` in root config.
- Programmatic access in config: use `loadEnv(mode, dir, prefixes)`.

```ts title=".env.production"
FARM_API_URL=https://api.example.com
```

```ts title="src/app.ts"
const url = import.meta.env.FARM_API_URL;
```

---

## Static Assets

- Files in `publicDir` (default: `public/`) are served at `/` in dev and copied to `output.path` in build.
- Import an asset in JS to get its URL string:

```ts
import logo from './logo.png';          // → URL string
import text from './readme.txt?raw';    // → raw string content
import url  from './img.png?url';       // → explicit URL
```

- Add custom extensions: `compilation.assets.include: ['txt', 'pdf']`.

---

## Source Maps

Set `compilation.sourcemap`:

| Value | Behavior |
|-------|----------|
| `true` (default) | Separate `.map` files, exclude node_modules |
| `false` | Disabled |
| `'inline'` | Inline in output, exclude node_modules |
| `'all'` | Separate files for all modules incl. node_modules |
| `'all-inline'` | Inline for all modules |

---

## Library Mode

Set `output.targetEnv: 'library'` to build a distributable library:

```ts title="farm.config.ts"
export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],           // array supported in library mode
      libraryBundleType: 'bundle-less', // 'single-bundle' | 'multiple-bundle' | 'bundle-less'
    },
  },
});
```

**Bundle types:**

| Type | Description | Multiple entries? |
|------|-------------|:-----------------:|
| `single-bundle` | All source merged into one file per format | No (single entry only) |
| `multiple-bundle` | Each entry gets its own bundle, shared code split | Yes |
| `bundle-less` | Each source module → its own output file (best tree-shaking for consumers) | Yes |

---

## Server-Side Rendering (SSR)

SSR requires **two builds**: one for the browser client, one for Node.js server.

**Typical project structure:**
```
index.html               # Client HTML entry (includes hydration placeholder)
farm.config.ts           # Browser build + dev server config
farm.config.server.ts    # Node build config
server.js                # Production server script (express/koa/etc.)
src/
  index-client.tsx       # Client entry — hydration
  index-server.tsx       # Server entry — renderToString
  main.tsx               # Shared app code
```

**`farm.config.server.ts`:**
```ts
export default defineConfig({
  compilation: {
    input:  { index: './src/index-server.tsx' },
    output: { targetEnv: 'node', format: 'cjs' },
  },
});
```

**Dev server in middleware mode (for SSR dev):**
```ts
// farm.config.ts
server: { middlewareMode: true }
```

SSR examples in repo: `examples/react-ssr`, `examples/vue-ssr`, `examples/solid-ssr`.

---

## Lazy Compilation

Enabled by default in dev mode. Modules are compiled on first request, drastically speeding up cold starts for large apps.

```ts
compilation: { lazyCompilation: true }  // default in dev; false in build
```

---

## Persistent Cache

Enabled by default. Caches compiled modules to `node_modules/.farm/cache`.  
Cache is automatically invalidated when `farm.config.ts` or its dependencies change.

```ts
compilation: { persistentCache: true }  // default; set false to disable
```

Clear cache: `farm clean` CLI command.

---

## Partial Bundling

Farm's default bundling strategy splits output into multiple resource chunks balanced between request count and file size — unlike full bundling (single file) or no bundling (many small files).

Key settings: `compilation.partialBundling.targetConcurrentRequests` (default: 25) and `targetMinSize` / `targetMaxSize`.

Use `groups` to hint which modules should be co-located, and `enforceResources` to force modules into the same output resource.

---

## HMR (Hot Module Replacement)

- Enabled automatically in dev mode (`server.hmr: true`).
- Works for JS, TS, JSX, TSX, CSS, and CSS Modules out of the box.
- Custom HMR handling via the [HMR API](https://farmfe.org/docs/api/hmr-api) (`import.meta.hot`).

```ts
if (import.meta.hot) {
  import.meta.hot.accept('./dep.ts', (newModule) => {
    // handle update
  });
}
```
