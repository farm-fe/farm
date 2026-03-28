# @farmfe/plugin-compress

🍣 A Farm plugin which compresses assets like JavaScript, CSS, and HTML files.

The plugin uses various compression algorithms to reduce the size of your assets, improving load times and performance. It supports multiple compression formats and can be configured to target specific file types.

Supported compression algorithms are:

- Brotli
- Gzip
- Deflate
- DeflateRaw

The default compression format is `brotli`.

## Requirements

This plugin requires an [LTS](https://github.com/nodejs/Release) Node version (v18.0.0+) and Farm v1.0.0+.

## Installation

```bash
npm i @farmfe/plugin-compress
```

## Usage

Create a `farm.config.js` [configuration file](https://www.farmfe.org/docs/config/configuring-farm) and import the plugin:

```js
import { defineConfig } from '@farmfe/core';
import compress from '@farmfe/plugin-compress';

export default defineConfig({
  plugins: [
    [
      compress()
    ]
  ],
});
```

Once the bundle is executed, the `console.log` will display the Base64 encoded representation of the image.

## Options

### `algorithm`

Type: `"gzip" | "brotli" | "deflateRaw" | "deflate"`<br>
Default: `"brotli"`

Specifies the compression algorithm to use.

### `level`

Type: `Number`<br>
Default: `6`

The compression level to apply. Higher values typically result in better compression but take more time.

### `threshold`

Type: `Number`<br>
Default: `1024`

The minimum size in bytes for a file to be compressed. Files smaller than this value will not be compressed.

### `filter`

Type: `String`<br>
Default: `'\\.(js|mjs|json|css|html)$'`

A regular expression string (not a RegExp object) that specifies which files should be compressed. The default matches common web asset extensions.

### `deleteOriginFile`

Type: `Boolean`<br>
Default: `false`

If `true`, the original uncompressed file will be deleted after successful compression.
