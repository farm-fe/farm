# Sass Plugin for Farm

Support compiling Sass/Scss in Farm.

## Usage

Install `@farmfe/js-plugin-sass` by your favorite package manager(npm, yarn, pnpm and so on):

```bash
npm i @farmfe/js-plugin-sass --save-dev # or pnpm/yarn add @farmfe/js-plugin-sass -D
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from '@farmfe/core/dist/config';
import Sass from '@farmfe/js-plugin-sass'; //  import the plugin

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
    // use the sass plugin.
    Sass({
      // custom options here
    }),
  ],
});
```

## Options

### match

Type: `string[]`

Default: `["\\.scss$"]`

Specifies the matching files.

### globals

Type: `string[]`

Default: `undefined`

Reads the contents from a file and injects them into each sass/scss file. It is typically used to inject some global variables

> Note that normal css should not be written in this file, otherwise it will inject them repeatedly into each compiled css file

### content

Type: `string`

Default: `undefined`

It has the same function as globals, but it can be conveniently used to inject some simple sass/scss content

### sourceMap

Type: `boolean`

Default: `false`

Whether to generate sourceMap

> If not set, it will read the compilation.sourcemap configuration in the farm configuration

### sassOption

Type: `StringOptions<'async'>`

Default: `{}`
