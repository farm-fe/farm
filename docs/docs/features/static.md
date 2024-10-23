# Static Assets

Farm treat modules that is not treated as `code` as `Static Assets`, for example, images like `png/svg/etc`, text files like `txt/xlsx/etc`. This document describes how Farm deal with these assets.

## url
Import a image：
```jsx
import rocketUrl from './assets/rocket.svg'; // return the url of this image

export function Main() {
  return <img src={rocketUrl} /> // using the url
}
```
Default to use url method when import a image. When using url methods to import a image, the image will be emitted to the output dir directly, and the image module itself will be compiled to a js module like:

```js
export default '/rocket.<content hash>.svg'
```
using [`compilation.output.assetFilename`](/docs/config/compilation-options#outputassetsfilename) to config your asset name。

## inline
Using query `?inline` to tell Farm that you want to inline your assets，then the assets will be transformed to base64，for example：

```js
// importer
import logo from './assets/logo.png?inline'; // logo is a base 64 str

// the image module will be compiled to:
export default 'data:image/png,base64,xxxxx==';
```

## raw
Using query `?raw` to tell Farm that you want to read the raw string of the assets, for example
```js
// import 
import logo from './assets/license.txt?raw'; // return the content string of the assets

// the txt file will be compiled to:
export default 'MIT xxxx';
```

## Configuring Assets
* Using [`compilation.output.assetFileName`](/docs/config/compilation-options#outputassetsfilename) to control the production file name
* using [`compilation.assets.include`](/docs/config/compilation-options#assetsinclude) to treat more kind of files as asset modules.

```js
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    output: {
      assetsFilename: 'assets/[resourceName].[hash].[ext]', // [] is a placeholder, Farm currently only these three kind of placeholders
    },
    assets: {
      include: ['txt'] // extra static asset extension
    }
  }
});
```
