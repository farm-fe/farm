# Vue Js Plugin for Farm

Support compiling Vue SFC in Farm.

## Usage

Install `@farmfe/js-plugin-vue` by your favorite package manager(npm, yarn, pnpm and so on):

```bash
npm i @farmfe/js-plugin-vue --save-dev # or pnpm/yarn add @farmfe/js-plugin-vue -D
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from '@farmfe/core/dist/config';
import farmJsPluginVue from '@farmfe/js-plugin-vue'; //  import the plugin

export default defineFarmConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
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

WIP.
