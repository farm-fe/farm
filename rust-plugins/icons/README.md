# @farmfe/plugin-icons

Access thousands of icons as components **on-demand** universally.

### Features

- 🌏 Universal
  - 🤹 **Any** icon sets - ~150 popular sets with over 200,000 icons, logos, emojis, etc. Powered by [Iconify](https://github.com/iconify/iconify).
  - 🚀 **Major** frameworks - Vanilla, Web Components, React, Vue 3, Solid, Svelte, and more. [Contribute](./src/compiler).
  - 🍱 **Any** combinations of them!
- ☁️ On-demand - Only bundle the icons you really use, while having all the options.
- 🌈 Stylable - Change size, color, or even add animations as you would with styles and classes.
- 📥 [Custom icons](#custom-icons) - load your custom icons to get universal integrations at ease.
- 🦾 TypeScript support.
- 🔍 [Browse Icons](https://icones.js.org/)

## Usage

Import icons names with the convention `~icons/{collection}/{icon}` and use them directly as components.

###### React

```jsx
import IconAccessibility from '~icons/carbon/accessibility'
import IconAccountBox from '~icons/mdi/account-box'

function App() {
  return (
    <div>
      <IconAccessibility />
      <IconAccountBox style={{ fontSize: '2em', color: 'red' }} />
    </div>
  )
}
```

###### Vue

```html
<script setup>
import IconAccessibility from '~icons/carbon/accessibility'
import IconAccountBox from '~icons/mdi/account-box'
</script>

<template>
  <icon-accessibility/>
  <icon-account-box style="font-size: 2em; color: red"/>
</template>
```

## Install

### Plugin

```bash
npm i -D @farmfe/plugin-icons
```

### Icons Data

We use [Iconify](https://iconify.design/) as the icons data source (supports 100+ iconsets).

You have two ways to install them:

###### Install Full Collection

```bash
npm i -D @iconify/json
```

`@iconify/json` (~120MB) includes all the iconsets from Iconify so you can install once and use any of them as you want (only the icons you actually use will be bundle into the production build).

###### Install by Icon Set

If you only want to use a few of the icon sets and don't want to download the entire collection, you can also install them individually with `@iconify-json/[collection-id]`.
For example, to install [Material Design Icons](https://icon-sets.iconify.design/mdi/), you can do:

```bash
npm i -D @iconify-json/mdi
```

To boost your workflow, it's also possible to let `unplugin-icons` handle that installation by enabling the `autoInstall` option.

```ts
Icons({
  // experimental
  autoInstall: true,
})
```

It will install the icon set when you import them. The right package manager will be auto-detected (`npm`, `yarn` or `pnpm`).

## Configuration

Create a `farm.config.js` [configuration file](https://www.farmfe.org/docs/config/configuring-farm) and import the plugin:

```ts
import { defineConfig } from '@farmfe/core';
import Icons from '@farmfe/plugin-plugin';

export default defineConfig({
  plugins: [
      ["@farmfe/plugin-icons", {
      /**
       * zie of zooming icon
       * @type {float}
       * @default 1.2
       */
      scale: 1.2,
      /**
       * @description Whether to automatically install the required dependencies
       * @type {boolean}
       * @default true
       */
      autoInstall: true,
      /**
       * @description The compiler used by the plugin
       * @type {string}
       * @default "jsx"
       * @enum ["jsx", "vue","react","preact","solid","svelte"]
       */
      compiler: "jsx",
      /**
       * @description The default style to apply to the svg element
       * @type {object}
       * @default {}
       */
      defaultStyle: {},
      /**
       * @description The default class to apply to the svg element
       * @type {string}
       */
      defaultClass: "",
      /**
       * @description Custom icon collection, support local svg and remote svg
       * @type {string}
       * @uses [iconname] to replace the icon name
       * @example
       *  import icon from "~icons/local/icon.svg"
       *  import icon from "~icons/remote/icon.svg"
       */
      customCollections: {
        local: './src/assets',
        remote: "https://cdn.simpleicons.org/[iconname]/"
      }
    }],
  ],
});
```

## Use RAW compiler from query params

From `v0.13.2` you can also use `raw` compiler to access the `svg` icon and use it on your html templates, just add `raw` to the icon query param.

For example, using `vue3`:

```vue
<script setup lang='ts'>
import RawMdiAlarmOff from '~icons/mdi/alarm-off?raw&width=4em&height=4em'
import RawMdiAlarmOff2 from '~icons/mdi/alarm-off?raw&width=1em&height=1em'
</script>

<template>
  <!-- raw example -->
  <pre>
    import RawMdiAlarmOff from '~icons/mdi/alarm-off?raw&width=4em&height=4em'
    {{ RawMdiAlarmOff }}
    import RawMdiAlarmOff2 from '~icons/mdi/alarm-off?raw&width=1em&height=1em'
    {{ RawMdiAlarmOff2 }}
  </pre>
  <!-- svg example -->
  <span v-html="RawMdiAlarmOff" />
  <span v-html="RawMdiAlarmOff2" />
</template>
```

## Custom Icons

you can now load your own icons!

```ts

Icons({
  customCollections: {
    'my-other-icons': "https://example.com/icons/[iconname].svg",
    // a helper to load icons from the file system
    // files under `./assets/icons` with `.svg` extension will be loaded as it's file name
    'my-yet-other-icons': './assets/icons',
  },
})
```

Then use as

```ts
import IconAccount from '~icons/my-icons/account'
import IconFoo from '~icons/my-other-icons/foo'
import IconBar from '~icons/my-yet-other-icons/bar'
```

## Icon customizer

you can also customize each icon using query params when importing them.

you can use `query` params to apply to individual icons:

<!-- eslint-skip -->

```vue
<script setup lang='ts'>
import MdiAlarmOff from 'virtual:icons/mdi/alarm-off?width=4em&height=4em'
import MdiAlarmOff2 from 'virtual:icons/mdi/alarm-off?width=1em&height=1em'
</script>

<template>
  <!-- width=4em and height=4em -->
  <mdi-alarm-off />
  <!-- width=4em and height=4em -->
  <MdiAlarmOff />
  <!-- width=1em and height=1em -->
  <MdiAlarmOff2 />
</template>
```
