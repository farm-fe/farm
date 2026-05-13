# Farm Plugins Reference

Source docs: `website/docs/using-plugins.mdx`, `website/docs/plugins/`

---

## Plugin Types Overview

| Type | Config Key | Language | Notes |
|------|-----------|----------|-------|
| Farm Rust Plugin | `plugins` (string) | Rust | Fastest; prefer when available |
| Farm JS Plugin | `plugins` (object) | JS/TS | Rollup-style hooks |
| Vite/Rollup/Unplugin | `vitePlugins` | JS | Compatible out of box |
| SWC Plugin | `compilation.script.plugins` | Rust/WASM | Per-module AST transforms |
| Runtime Plugin | `compilation.runtime.plugins` | JS | Module system extensions |

---

## Using Farm Rust Plugins

Configure by package name (string) in the `plugins` array:

```ts title="farm.config.ts"
export default defineConfig({
  plugins: [
    '@farmfe/plugin-sass',                             // string = Rust plugin
    ['@farmfe/plugin-sass', { additionalData: '' }],   // [name, options] = with config
  ],
});
```

---

## Using Farm JS Plugins

Configure as objects returned by plugin factory functions:

```ts title="farm.config.ts"
import farmPostcss from '@farmfe/js-plugin-postcss';

export default defineConfig({
  plugins: [
    farmPostcss({ /* postcss options */ }),
    // inline plugin:
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
  ],
});
```

> **`filters` are required** on every per-module hook in JS plugins — unmatched modules are skipped on the Rust side for performance.

Plugin priority: larger value = earlier execution. Internal plugins run at `99`. Set `priority > 99` to run before them, `< 99` to run after.

---

## Using Vite / Rollup / Unplugin Plugins

Configure in `vitePlugins` (not `plugins`):

```ts title="farm.config.ts"
import vue from '@vitejs/plugin-vue';
import AutoImport from 'unplugin-auto-import/vite';

export default defineConfig({
  vitePlugins: [
    vue(),
    AutoImport({ /* options */ }),
    // Function form — adds filters for performance:
    () => ({ vitePlugin: vue(), filters: ['\\.vue$'] }),
  ],
});
```

---

## Using SWC Plugins

Configure in `compilation.script.plugins`:

```ts title="farm.config.ts"
export default defineConfig({
  compilation: {
    script: {
      plugins: [{
        name: 'swc-plugin-vue-jsx',
        options: { transformOn: true },
        filters: { moduleTypes: ['tsx', 'jsx'] },
      }],
    },
  },
});
```

Each item has:
- `name` — npm package name of the SWC plugin
- `options` — options passed to the plugin
- `filters` — `resolvedPaths` and/or `moduleTypes` (union when both specified)

---

## Using Runtime Plugins

Configure in `compilation.runtime.plugins` (absolute paths recommended):

```ts title="farm.config.ts"
import path from 'path';

export default defineConfig({
  compilation: {
    runtime: {
      plugins: [
        require.resolve('farm-plugin-runtime-mock'),
        path.join(process.cwd(), 'build/runtime-plugin.ts'),
      ],
    },
  },
});
```

---

## Official Rust Plugins

| Package | Description | Install |
|---------|-------------|---------|
| `@farmfe/plugin-react` | React JSX + react-refresh | `pnpm add -D @farmfe/plugin-react` |
| `@farmfe/plugin-sass` | Sass/SCSS (sass-embedded) | `pnpm add -D @farmfe/plugin-sass` |
| `@farmfe/plugin-strip` | Remove debugger/console/assert | `pnpm add -D @farmfe/plugin-strip` |
| `@farmfe/plugin-dsv` | `.csv`/`.tsv` → JS modules | `pnpm add -D @farmfe/plugin-dsv` |
| `@farmfe/plugin-yaml` | YAML → ES modules | `pnpm add -D @farmfe/plugin-yaml` |
| `@farmfe/plugin-virtual` | Virtual modules | `pnpm add -D @farmfe/plugin-virtual` |
| `@farmfe/plugin-react-components` | Auto-import React components | `pnpm add -D @farmfe/plugin-react-components` |

## Official JS Plugins

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

---

## Common Plugin Configurations

**React:**
```ts
plugins: ['@farmfe/plugin-react']
```

**Vue 3 (via Vite plugin):**
```ts
import vue from '@vitejs/plugin-vue';
export default defineConfig({ vitePlugins: [vue()] });
```

**Sass with global variables:**
```ts
plugins: [['@farmfe/plugin-sass', { additionalData: '@use "./global.scss";' }]]
```

**PostCSS / TailwindCSS:**
```ts
import postcss from '@farmfe/js-plugin-postcss';
export default defineConfig({ plugins: [postcss()] });
// tailwind config lives in postcss.config.js / tailwind.config.js
```

**Less:**
```ts
import less from '@farmfe/js-plugin-less';
export default defineConfig({ plugins: [less()] });
```

---

## Community Plugins

See https://github.com/farm-fe/awesome-farm for community-maintained plugins.
