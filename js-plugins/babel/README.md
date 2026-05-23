# @farmfe/js-plugin-babel

farm plugin which support use babel

## Install

```shell
npm i @farmfe/js-plugin-babel -D
```

or yarn/pnpm

```shell
pnpm i @farmfe/js-plugin-babel -D
```

## Usage

```ts
// farm.config.ts
import { babel } from "@farmfe/js-plugin-babel";
import react from "@farmfe/plugin-react";

defineConfig({
  plugins: [
    // transform by babel, default transform `js`, `jsx`, `ts`, `tsx` files
    babel(),
    // transform react
    react(),
  ],
});
```

## Options

### `filters`

- Type: `{ moduleTypes: ModuleType[], resolvedPaths: string[] }`
- Default:

```ts
{
    moduleTypes: ["js", "jsx", "ts", "tsx"],
    resolvedPaths: []
}
```

Determines which files to transform

For example, files with the `tsx` extension

```ts
{
  resolvedPaths: [".tsx$"];
}
```

Or use module types to distinguish

```ts
{
  moduleTypes: ["tsx", "jsx"];
}
```

The type comes from the return value of the `load` hook and can be customized (`farm` has some default types `js`, `jsx`, `ts`, `tsx`, `css`, `html`, `asset`, `runtime` that can be used directly)

### `transformOptions`

Babel [transform configuration](https://babeljs.io/docs/options)

### `priority`

- Type: `number`
- Default: `99`

The priority of the farm plugin execution, the higher the priority, the earlier it executes.

### `name`

- Type: `string`
- Default: `js-plugin:babel`

The name of the farm plugin
