# @farmfe/plugin-modular-import

Modular UI library build plugin for Farm.

## Install

### Plugin

```bash
npm i -D @farmfe/plugin-modular-import
```

### Usage

Via `farm.config.ts`.

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  plugins: [
      ["@farmfe/plugin-modular-import", {
        /**
         * zie of zooming icon
         * @type {string}
         * @default lib
         */
        libDir: 'lib',
        /**
         * @description The components lib directory
         * @type {string}
         */
        libraryName: "",
        /**
         * @description The UI library name
         * @type {boolean}
         * @default true
         */
        camel2Dash: true,
        /**
         * @description style lib directory, default "lib"
         * @type {string}
         * @default lib
         */
        styleLibDir: 'lib',
        /**
         * @description the style library name. e.g. custon-theme =>  custon-theme/index.css
         * @type {string}
         */
        styleLibraryName: '',
        /**
         * @description custom style path
         * @type {string}
         * @default index.css
         */
        styleLibraryPath: 'index.css',
    }],
  ],
});
```

### Example

#### Default Usage

```ts
export default defineConfig({
  plugins: [
    ['@farmfe/plugin-modular-import', {
      libraryName: 'element-ui',
    }]
  ],
});
```

###### Converts

```js
import { SomeComponent } from 'element-ui'
```

###### To

```js
import SomeComponent from 'element-ui/lib/SomeComponent';
import 'element-ui/lib/SomeComponent/index.css';
```

#### Set `libDir` Usage

```ts
export default defineConfig({
  plugins: [
    ['@farmfe/plugin-modular-import', {
      libraryName: 'element-ui',
      libDir: 'es',
    }]
  ],
});
```

###### Converts

```js
import { SomeComponent } from 'element-ui'
```

###### To

```js
import SomeComponent from 'element-ui/es/SomeComponent';
import 'element-ui/lib/SomeComponent/index.css';
```

#### Set `camel2Dash` Usage

```ts
export default defineConfig({
  plugins: [
    ['@farmfe/plugin-modular-import', {
      libraryName: 'element-ui',
      libDir: 'es',
      camel2Dash: false,
    }]
  ],
});
```

###### Converts

```js
import { SomeComponent } from 'element-ui'
```

###### To

```js
import SomeComponent from 'element-ui/es/someComponent';
import 'element-ui/lib/someComponent/index.css';
```

#### Set `styleLibDir` Usage

```ts
export default defineConfig({
  plugins: [
    ['@farmfe/plugin-modular-import', {
      libraryName: 'element-ui',
      libDir: 'es',
      camel2Dash: false,
      styleLibDir: 'lib',
    }]
  ],
});
```

###### Converts

```js
import { SomeComponent } from 'element-ui'
```

###### To

```js
import SomeComponent from 'element-ui/es/someComponent';
import 'element-ui/lib/someComponent/index.css';
```

#### Set `styleLibraryName` Usage

```ts
export default defineConfig({
  plugins: [
    ['@farmfe/plugin-modular-import', {
      libraryName: 'element-ui',
      libDir: 'es',
      camel2Dash: false,
      styleLibDir: 'lib',
      styleLibraryName: 'theme-default',
    }]
  ],
});
```

###### Converts

```js
import { SomeComponent } from 'element-ui'
```

###### To

```js
import SomeComponent from 'element-ui/es/someComponent';
import 'element-ui/lib/theme-default/someComponent/index.css';
```

#### Set `styleLibraryPath` Usage

```ts
export default defineConfig({
  plugins: [
    ['@farmfe/plugin-modular-import', {
      libraryName: 'element-ui',
      libDir: 'es',
      camel2Dash: false,
      styleLibDir: 'lib',
      styleLibraryName: 'theme-default',
      styleLibraryPath: 'style/index.css'
    }]
  ],
});
```

###### Converts

```js
import { SomeComponent } from 'element-ui'
```

###### To

```js
import SomeComponent from 'element-ui/es/someComponent';
import 'element-ui/lib/theme-default/someComponent/style/index.css';
```