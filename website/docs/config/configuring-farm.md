# Configuring Farm

## Config File Spec
By default, Farm reads the first config file it finds in the project root, in this order:

`farm.config.ts`, `farm.config.js`, `farm.config.cjs`, `farm.config.mjs`, `farm.config.cts`, `farm.config.mts`.

An example configuration file:

```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  root: process.cwd(), // compiled root directory
  // compile options
  compilation: {
    //...
  },
  // Dev Server options
  server: {
    hmr: true,
    //...
  },
  // plugin configuration
  plugins: [],
});
```

For config options details, refer to:
* [`Compiler Options`](/docs/config/compilation-options): Configuring compiler options(`compilation` field), like `input`, `output`, `css compilation`, `bundling rules` and so on.
* [`Dev Server Options`](/docs/config/dev-server): Configuring dev server options(`server` field), like `port`, `host`, `protocol` and so on.
* [`Shared Options`](/docs/config/shared): Configuring shared options between `compiler options` and `dev server options`, like `root`, `env` and so on.

:::note
You can also use `farm start/build -c my-config.ts` (or `--config my-config.ts`) to use a custom config file.
:::

## Config Resolution and Validation

Farm resolves `--config` relative to the project root. If no explicit config file is passed, Farm searches the project root using the filename order above.

Configuration files are validated before Farm starts. Unknown fields are rejected, so prefer the documented `UserConfig` shape rather than passing internal compiler fields directly.

When loading TypeScript or ESM config files, Farm bundles the config and imports the generated file from `node_modules/.farm/`. The output format is inferred from the config extension (`.cjs`/`.cts` use CommonJS; `.js`/`.mjs`/`.mts` use ESM). For uncommon loader interop cases, `FARM_CONFIG_FORMAT=cjs` or `FARM_CONFIG_FORMAT=esm` can override that inference.

## Loading Ts Config File
Farm supports TypeScript config files like `farm.config.ts`, `farm.config.cts`, and `farm.config.mts` out of the box. Farm bundles the config file and its local dependencies into `node_modules/.farm/` first, then imports the generated bundle.

Config files may export an object, a promise, or a function that receives `{ mode, command, isPreview }`:

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig(({ mode, command }) => ({
  compilation: {
    mode,
  },
}));
```

Farm replaces `__dirname` and `__filename` in bundled config files with the original config file directory and filename. You can also use standard ESM APIs such as `import.meta.url` in ESM config files.

Or you can use `farm.config.mjs` or `farm.config.cjs` with `@type` for editor type support:

```js title="farm.config.mjs"
/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  // ...
}
```

## Examples
### Input and Output
```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  // compile options
  compilation: {
    input: {
      index: './src/index.html',
      about: './src/about.html',
    },
    output: {
      path: 'build',
      publicPath: process.env.NODE_ENV === 'production' ? 'https://my-cdn.com' : '/'
    }
  },
});
```

In above example, we configured `./src/index.html` and `./src/about.html` as input, then output the compiled resources to `build` dir.

### Dev Server Port

```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  server: {
    port: 9801
  }
});
```

### Disable Default Optimizations
```ts title="farm.config.ts" {5-7}
import { defineConfig } from "@farmfe/core";

export default defineConfig({
  // compile options
  compilation: {
    lazyCompilation: false,
    persistentCache: false,
    minify: false,
    treeShaking: false
  },
});
```
