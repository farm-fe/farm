# Migrate From Vite

Farm supports many Vite-style project conventions, but it is not a drop-in replacement for every Vite internal API. Treat migration as a config and plugin compatibility pass.

## 1. Rename the config file

Rename `vite.config.ts` to `farm.config.ts` and import Farm's helper:

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  // Farm config
});
```

Refer to [Configuring Farm](/docs/config/configuring-farm) for current Farm config fields.

## 2. Move Vite plugins to `vitePlugins`

Farm's native plugins use `plugins`. Vite-compatible plugins belong in `vitePlugins`:

```ts
import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  vitePlugins: [vue()],
  plugins: ['@farmfe/plugin-sass']
});
```

Some Vite plugins depend on Vite internals and may not work in Farm. When a plugin is tightly coupled to Vite, prefer a Farm official plugin, a PostCSS plugin, or a framework-specific integration instead.

## 3. Remove Vite-only options

Many Vite optimization options are not needed in Farm. Start by removing Vite-only fields such as `optimizeDeps`, then add Farm equivalents only when needed.

Common replacements:

| Vite pattern | Farm guidance |
| --- | --- |
| `plugins` for Vite plugins | Move to `vitePlugins`. |
| Asset/base path | Use Farm `compilation.output.publicPath` or CLI `--base`. |
| Dev server settings | Use Farm `server`. |
| SSR-specific Vite config | Review [Farm SSR](/docs/advanced/ssr). |

## 4. Verify the migration

Run both dev and production builds:

```bash
npx farm dev
npx farm build
```

If a Vite plugin fails during migration, temporarily remove it and re-add integrations one by one so you can distinguish config issues from plugin compatibility issues.

We have migrated a [real Vite admin project](https://github.com/farm-fe/farm-soybean-admin) to Farm; use it as a reference for practical migration patterns.
