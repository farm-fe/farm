# SolidJS support for Farm

Support SolidJS in Farm.

## Usage

Install `@farmfe/js-plugin-solid` by your favorite package manager (npm, yarn, pnpm and so on):

```bash
npm i @farmfe/js-plugin-solid --save-dev # or pnpm/yarn add @farmfe/js-plugin-solid -D
```

Configuring the plugin in `farm.config.ts`:

```ts
import solid from '@farmfe/js-plugin-solid';

export default {
  compilation: {
    // ...
  },
  plugins: [solid()]
};
```

## Options

see [vite-plugin-solid options](https://github.com/solidjs/vite-plugin-solid#api)
