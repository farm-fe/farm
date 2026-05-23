# @farmfe/plugin-image

🍣 A Farm plugin which imports JPG, PNG, GIF, SVG, and WebP files.

Images are encoded using base64, which means they will be 33% larger than the size on disk. You should therefore only use this for small images where the convenience of having them available on startup (e.g. rendering immediately to a canvas without co-ordinating asynchronous loading of several images) outweighs the cost.

## Requirements

This plugin requires an [LTS](https://github.com/nodejs/Release) Node version (v18.0.0+) and Farm v1.0.0+.

## Installation

```bash
npm i @farmfe/plugin-image
```

## Usage

Create a `farm.config.js` [configuration file](https://www.farmfe.org/docs/config/configuring-farm) and import the plugin:

```js
import { defineConfig } from '@farmfe/core';
import image from '@farmfe/plugin-image';

export default defineConfig({
  plugins: [
    [
      image()
    ]
  ],
});
```

Once the bundle is executed, the `console.log` will display the Base64 encoded representation of the image.

## Options

### `dom`

Type: `Boolean`<br>
Default: `false`

If `true`, instructs the plugin to generate an ES Module which exports a DOM `Image` which can be used with a browser's DOM. Otherwise, the plugin generates an ES Module which exports a `default const` containing the Base64 representation of the image.

Using this option set to `true`, the export can be used as such:

```js
import logo from './rollup.png';
document.body.appendChild(logo);
```

### `exclude`

Type: `String` | `Array[...String]`<br>
Default: `null`

A [picomatch pattern](https://github.com/micromatch/picomatch), or array of patterns, which specifies the files in the build the plugin should _ignore_. By default no files are ignored.

### `include`

Type: `String` | `Array[...String]`<br>
Default: `null`

A [picomatch pattern](https://github.com/micromatch/picomatch), or array of patterns, which specifies the files in the build the plugin should operate on. By default all files are targeted.
