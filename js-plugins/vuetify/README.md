# @farmfe/js-plugin-vuetify

Farm plugin for [Vuetify](https://vuetifyjs.com/), a component framework for Vue 3.

## Install

```shell
npm i @farmfe/js-plugin-vuetify -D
```

or yarn/pnpm

```shell
pnpm add @farmfe/js-plugin-vuetify -D
```

## Usage

```ts
// farm.config.ts
import vuetify from "@farmfe/js-plugin-vuetify";
import vue from "unplugin-vue/vite";

defineConfig({
  plugins: [vue(), vuetify()],
});
```

## Options

Vuetify plugin options are re-exported from `@vuetify/loader-shared`.

### `autoImport`

> Type annotations defined in `@vuetify/loader-shared`:
>
> ```ts
> interface ObjectImportPluginOptions {
>   labs?: boolean;
>   ignore?: (keyof typeof Components | keyof typeof Directives)[];
> }
> type ImportPluginOptions = boolean | ObjectImportPluginOptions;
> ```

- Type: `ImportPluginOptions`
- Default: `true`

Whether to automatically import Vuetify components and directives.

### `styles`

- Type:

```ts
true | 'none' | 'sass' | {
  configFile: string;
};
```

- Default: `true`

Determine how to import the Vuetify styles.
