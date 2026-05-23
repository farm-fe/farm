# @farmfe/plugin-url

🍣 A farm plugin which imports files as data-URIs or ES Modules.

## Requirements

This plugin requires an [LTS](https://github.com/nodejs/Release) Node version (v18.0.0+) and farm v1.0.0+.

## Install

Using npm:

```console
npm install @farmfe/plugin-url --save-dev
```

## Usage

Create a `farm.config.js` [configuration file](https://www.farmfe.org/docs/config/configuring-farm) and import the plugin:

```typescript
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: [
    [
      '@farmfe/plugin-url',
      {},
    ],
  ],
});
```

With an accompanying file `src/index.js`, the local `image.svg` file would now be importable as seen below:

```js
// src/index.js
import svg from './image.svg';
console.log(`svg contents: ${svg}`);
```

## Options

### `exclude`

Type: `String` | `Array[...String]`<br>
Default: `null`

A [picomatch pattern](https://github.com/micromatch/picomatch), or array of patterns, which specifies the files in the build the plugin should _ignore_. By default no files are ignored.

### `include`

Type: `String` | `Array[...String]`<br>
Default: `[".*\.svg$",".*\.png$",".*\.jp(e)?g$", ".*\.gif$", ".*\.webp$",]`

A [picomatch pattern](https://github.com/micromatch/picomatch), or array of patterns, which specifies the files in the build the plugin should operate on. By default .svg, .png, .jpg, .jpeg, .gif and .webp files are targeted.

### `limit`

Type: `Number`<br>
Default: `14336` (14kb)

The file size limit for inline files. If a file exceeds this limit, it will be copied to the destination folder and the hashed filename will be provided instead. If `limit` is set to `0` all files will be copied.

### `publicPath`

Type: `String`<br>
Default: (empty string)

A string which will be added in front of filenames when they are not inlined but are copied.

### `emitFiles`

Type: `Boolean`<br>
Default: `true`

If `false`, will prevent files being emitted by this plugin. This is useful for when you are using Rollup to emit both a client-side and server-side bundle.

### `fileName`

Type: `String`<br>
Default: `'[hash][extname]'`

If `emitFiles` is `true`, this option can be used to rename the emitted files. It accepts the following string replacements:

- `[hash]` - The hash value of the file's contents
- `[name]` - The name of the imported file (without its file extension)
- `[extname]` - The extension of the imported file (including the leading `.`)
- `[dirname]` - The parent directory name of the imported file (including trailing `/`)

### sourceDir

Type: `String`<br>
Default: (empty string)

When using the `[dirname]` replacement in `fileName`, use this directory as the source directory from which to create the file path rather than the parent directory of the imported file. For example:

_src/path/to/file.js_

```js
import png from './image.png';
```

_rollup.config.js_

```js
url({
  fileName: '[dirname][hash][extname]',
  sourceDir: path.join(__dirname, 'src')
});
```

Emitted File: `path/to/image.png`

### `destDir`

Type: `String`<br>
Default: (empty string)

The destination dir to copy assets, usually used to rebase the assets according to HTML files.
