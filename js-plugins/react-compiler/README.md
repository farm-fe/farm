# @farmfe/js-plugin-react-compiler

farm plugin which support use react-compilers

## Install

```shell
npm i @farmfe/js-plugin-react-compiler -D
```

or yarn/pnpm

```shell
pnpm i @farmfe/js-plugin-react-compiler -D
```

## Usage

```ts
// farm.config.ts
import { reactCompiler } from "@farmfe/js-plugin-react-compiler";
import react from "@farmfe/plugin-react";

defineConfig({
  plugins: [
    // transform by babel & react compiler, default transform `tsx`, `jsx` files
    reactCompiler(),
    // transform jsx
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
    moduleTypes: ["tsx", "jsx"],
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

### `compilerOptions`

react compiler babel config, see [here](https://github.com/facebook/react/blob/main/compiler/packages/babel-plugin-react-compiler/src/Entrypoint/Options.ts#L39)
