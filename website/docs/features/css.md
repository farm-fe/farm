# Css/Sass/Less
Farm support Css out of box, just import the css file:

```tsx
import './index.css';
```

Then farm will auto enable HMR for css module, and generating bundled resources for css.

## Css Modules
Farm support css modules out of box, the modules end with `.module.css|less|scss|sass` will be treated as css modules by default.

```tsx title="comp.tsx"
import styles from './index.module.css'

export function Comp() {
  return <div className={styles.main}>Main</div>
}
```
```css title="index.module.css"
.main {
  color: green;
}
```
You can configuring css modules by [`css.modules`](/docs/config/compilation-options#cssmodules). for example you can set `css.modules.paths` to `['.css|sass|less|scss']` then all css files will be treated as css modules.

### CSS Modules Locals Conversion
Farm supports converting the naming convention of CSS module class names via [`css.modules.localsConversion`](/docs/config/compilation-options#cssmoduleslocalsconversion). Available modes:

- `asIs` — Class names are exported as-is (default)
- `camelCase` — Class names are converted to camelCase
- `camelCaseOnly` — Class names are converted to camelCase and the original name is removed
- `dashes` — Only dashes in class names are converted to camelCase
- `dashesOnly` — Only dashes are converted, and the original name is removed

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    css: {
      modules: {
        localsConversion: 'camelCase',
      }
    }
  }
})
```

### Transform CSS to Script
Farm supports transforming CSS into JavaScript modules via [`css.transformToScript`](/docs/config/compilation-options#csstransformtoscript). When enabled, CSS modules will be compiled into JS that exports the class name mappings and injects styles at runtime.

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    css: {
      transformToScript: true,
    }
  }
})
```

## Css Pre-Processor
Farm provide official sass, less, postcss plugins to support css pre-processor.

### Sass
Farm sass plugin is a Rust Plugin and use `sass-embeded`(we may migrate to [grass](https://github.com/connorskees/grass) in the future).

Steps to compile `sass/scss` modules in Farm.
1. Install dependencies
```sh
# npm or yarn or pnpm, choose your favorite package manager
pnpm add -D @farmfe/plugin-sass
```

2. Configure the plugin
```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  // ...
  plugins: ['@farmfe/plugin-sass'] // to use a rust plugin, just configure its package name as a string
  // if you want to specify options for plugin-sass, use
  // plugins: [
  //   ['@farmfe/plugin-sass', { sourceMap: false }]
  // ]
});
```

3. Import sass module
```ts
import './index.scss';
```

To use sass with css modules, change the file name from `index.scss` to `index.module.scss`, see [css modules](/docs/config/farm-config#cssmodules).

`@farmfe/plugin-sass` supports a lot of options, use the array syntax of `plugins` to specify options for plugin sass:

```ts
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  // if you want to specify options for plugin-sass, use
  plugins: [
    [
      '@farmfe/plugin-sass',
      // all supported options as below
      {
        sourceMap: true // bool
        sourceMapIncludeSources: true, // bool
        alertAscii: true, // bool
        alertColor: true, // bool
        charset: true, // bool
        quietDeps: true, // bool
        verbose: false, // bool
        style: 'expanded' | 'compressed' // output code style
      }
    ]
  ]
});
```


### Less
Farm less plugin is a Js Plugin. Steps to compile `less` modules in Farm.

1. Install dependencies
```sh
# npm or yarn or pnpm, choose your favorite package manager
pnpm add -D @farmfe/js-plugin-less
```

2. Configure the plugin
```ts
import { defineConfig } from '@farmfe/core';
import less from '@farmfe/js-plugin-less';

export default defineConfig({
  // ...
  plugins: [less()] // pass argument to the less function like `less({ /* your options */ })` to specify less options
});
```

3. Import sass module
```ts
import './index.less';
```

To use sass with css modules, change the file name from `index.less` to `index.module.less`, see [css modules](/docs/config/farm-config#cssmodules)

### Postcss
The Farm postcss plugin is a JS plugin. The steps to introduce postcss in Farm are as follows:

1. Install dependencies
```sh
# npm or yarn or pnpm, choose your favorite package manager
pnpm add -D @farmfe/js-plugin-postcss
```

2. Configure the plugin
```ts
import { defineConfig } from '@farmfe/core';
import postcss from '@farmfe/js-plugin-postcss';

export default defineConfig({
   //...
   plugins: [postcss()] // pass argument to the less function like `less({ /* your options */ })` to specify less options
});
```

3. Configure `postcss.config.js` and import the required postcss plugins

```js title=postcss.config.js
module.exports = {
   plugins: [
     require('postcss-pxtorem')({
       rootValue: 16,
       propList: ['*'],
     }),
   ]
}
```

### TailwindCSS

Farm provides a dedicated `@farmfe/js-plugin-tailwindcss` plugin for **TailwindCSS v4** integration. It uses the native Tailwind v4 compiler directly — no `postcss.config.js` is needed.

1. Install dependencies
```sh
pnpm add -D @farmfe/js-plugin-tailwindcss tailwindcss
```

2. Configure the plugin
```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';
import tailwindcss from '@farmfe/js-plugin-tailwindcss';

export default defineConfig({
  plugins: [tailwindcss()],
});
```

3. Add the Tailwind directive to your CSS entry file
```css title="index.css"
@import "tailwindcss";
```

4. Import the CSS file in your entry
```ts title="main.ts"
import './index.css';
```

That's it — Farm will scan your source files for Tailwind utility classes and generate the corresponding CSS automatically.

:::tip
`@farmfe/js-plugin-tailwindcss` targets **TailwindCSS v4**. If you need TailwindCSS v3, use `@farmfe/js-plugin-postcss` with `require('tailwindcss')` in your `postcss.config.js` instead.
:::

#### Filtering scanned files

By default the plugin scans all files except those inside `node_modules`. You can restrict which files are scanned for utility classes via the `filters` option:

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';
import tailwindcss from '@farmfe/js-plugin-tailwindcss';

export default defineConfig({
  plugins: [
    tailwindcss({
      filters: {
        resolvedPaths: ['src/'],
      },
    }),
  ],
});
```

## Css Prefixer
Farm supports css prefixer out of box, you can configure it using `compilation.css.prefixer`.

:::note
`css.prefix.targets` will be set automatically when [`output.targetEnv`](/docs/config/compilation-options#output-targetenv). Normally set [`output.targetEnv`](/docs/config/compilation-options#output-targetenv) would be enough.
:::

```ts title="farm.config.ts"
import { defineConfig } from '@farmfe/core';

export default defineConfig({
   compilation: {
     css: {
       prefix: {
        targets: ['ie >= 10']
       }
     },
   },
});
```
Then for input code:
```css
div {
  display: flex;
}
```
output code:
```css
div{display:-ms-flexbox;display:flex}
```
