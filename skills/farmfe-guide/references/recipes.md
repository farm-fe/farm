# Farm Recipes & Troubleshooting

Source docs: `website/docs/`, `website/docs/frameworks/`

---

## Common Config Patterns

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

### Multiple HTML Entries (MPA)

```ts
export default defineConfig({
  compilation: {
    input: { index: './index.html', admin: './admin.html' },
  },
  server: { appType: 'mpa' },
});
```

### Vendor Bundle Group

```ts
compilation: {
  partialBundling: {
    groups: [{ name: 'vendor', test: ['node_modules/'] }],
  },
}
```

### Disable All Optimizations (dev debugging)

```ts
compilation: {
  lazyCompilation: false,
  persistentCache: false,
  minify:          false,
  treeShaking:     false,
}
```

### HTTPS Dev Server

```ts
import fs from 'fs';
export default defineConfig({
  server: {
    https: {
      key:  fs.readFileSync('./localhost.key'),
      cert: fs.readFileSync('./localhost.crt'),
    },
  },
});
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

### Node.js / SSR Build

```ts
export default defineConfig({
  compilation: {
    input:  { server: './src/server.ts' },
    output: { targetEnv: 'node', format: 'cjs' },
  },
});
```

### Library — Dual ESM + CJS Output

```ts
export default defineConfig({
  compilation: {
    input: { index: './src/index.ts' },
    output: {
      targetEnv: 'library',
      format: ['esm', 'cjs'],
      libraryBundleType: 'bundle-less',
    },
  },
});
```

---

## Framework Quick Configs

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

### Vue 3 + JSX

```ts
import vue from '@vitejs/plugin-vue';
import vueJsx from '@vitejs/plugin-vue-jsx';
export default defineConfig({
  vitePlugins: [vue(), vueJsx()],
});
```

### Svelte

```ts
import { svelte } from '@sveltejs/vite-plugin-svelte';
export default defineConfig({
  vitePlugins: [svelte()],
});
```

### Solid

```ts
import solidPlugin from 'vite-plugin-solid';
export default defineConfig({
  vitePlugins: [solidPlugin()],
});
```

### Preact

```ts
import preact from '@preact/preset-vite';
export default defineConfig({
  vitePlugins: [preact()],
});
```

### Electron

```ts
import electron from '@farmfe/js-plugin-electron';
export default defineConfig({
  plugins: [electron({ /* options */ })],
});
```

### NestJS

```ts
export default defineConfig({
  compilation: {
    input:  { main: './src/main.ts' },
    output: { targetEnv: 'node', format: 'cjs' },
  },
});
```

---

## Troubleshooting

| Symptom | Fix |
|---------|-----|
| Stale cache / unexpected build errors | `farm clean` or set `compilation.persistentCache: false` |
| Port conflict on startup | Set `server.strictPort: false` or change `server.port` |
| `__dirname` / `__filename` undefined in config | Use `import.meta.url` → `new URL('./...', import.meta.url).pathname` |
| Symlinked deps not resolved (pnpm) | Ensure `compilation.resolve.symlinks: true` (required for pnpm) |
| Type-only imports causing runtime errors | Set `compilation.script.importNotUsedAsValues: 'remove'` |
| Large bundle / too many polyfills | Avoid `browser-legacy` unless IE is required; use `browser-es2017` instead |
| SWC plugin version incompatibility | Pin the plugin to match Farm's `swc_core` version; check the plugin's changelog |
| Vite plugin not applying | Use `vitePlugins` array, not `plugins` |
| CSS changes not hot-reloading | Ensure `server.hmr: true`; set `css.transformToScript: true` in dev for per-module HMR |
| Cannot resolve package in monorepo | Add the duplicate to `compilation.resolve.dedupe: ['react']` |
| Config TypeScript errors | Import types: `import type { UserConfig } from '@farmfe/core'` |

---

## External Links

- **Docs**: https://farmfe.org/docs
- **Config Reference**: https://farmfe.org/docs/config/compilation-options
- **Plugin API**: https://farmfe.org/docs/api/js-plugin-api
- **JS API**: https://farmfe.org/docs/api/javascript-api
- **HMR API**: https://farmfe.org/docs/api/hmr-api
- **Awesome Farm (community plugins)**: https://github.com/farm-fe/awesome-farm
- **Examples in this repo**: `examples/` directory
- **Source docs**: `website/docs/` directory
