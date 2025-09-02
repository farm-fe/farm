<div align="center">
  <a href="https://github.com/farm-fe/farm">
  <img src="../../assets/logo.png" width="550" />
  </a>
  <p>
    <span>English</span> |
    <a href="https://github.com/farm-fe/farm/blob/main/js-plugins/sass/README-zh-CN.md">简体中文</a>  
</div>

---

# Sass Plugin for Farm

Support compiling Sass/Scss in Farm.

## Usage

Install `@farmfe/js-plugin-sass` by your favorite package manager(npm, yarn, pnpm and so on):

```bash
npm i @farmfe/js-plugin-sass --save-dev # or pnpm/yarn add @farmfe/js-plugin-sass -D
```

Configuring the plugin in `farm.config.ts`:

```ts
import { defineFarmConfig } from 'farm/dist/config';
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

### implementation

Type: `string | undefined`

Default: `undefined`

Specify the executor of the sass file (such as sass, sass-embedded), if not defined, the file in node_module will be searched by default.

### match

Type: `string[]`

Default: `["\\.s[ac]ss$"]`

Specifies the matching files.

### globals

Type: `string[]`

Default: `[]`

Reads the contents from a file and injects them into each sass/scss file. It is typically used to inject some global variables

> Note that normal css should not be written in this file, otherwise it will inject them repeatedly into each compiled css file


### content

Type: `string | undefined`

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
