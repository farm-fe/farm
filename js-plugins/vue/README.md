# Vue Js Plugin for Farm

Support compiling Vue SFC in Farm.

## Usage

Install `@farmfe/js-plugin-vue` by your favorite package manager(npm, yarn, pnpm and so on):

```bash
npm i @farmfe/js-plugin-vue --save-dev # or pnpm/yarn add @farmfe/js-plugin-vue -D
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from "farm/dist/config";
import farmJsPluginVue from "@farmfe/js-plugin-vue"; //  import the plugin

export default defineFarmConfig({
  compilation: {
    input: {
      index: "./index.html",
    },
    output: {
      path: "./build",
    },
  },
  plugins: [
    // use the vue plugin.
    farmJsPluginVue({
      // custom options here
    }),
  ],
});
```

## Options

### hmr

- Type: `boolean`
- Default: `false`

Determine whether to enable `hot module replace`

If not set, it is considered to be `true` in `development` mode.

### ssr

- Type: `boolean`
- Default: `false`

When set to `true`, it will disable `compilation.lazyCompilation` and `server.hmr`.
