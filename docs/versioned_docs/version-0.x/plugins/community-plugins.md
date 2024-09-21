# Community Plugins
Farm support `Vite/Rollup` plugins out of box. So `Vite/Rollup` or `unplugin` plugins can be used in Farm directly.

:::tip
PR welcome if you developed a Farm compatible plugin and you want to list it here.
:::

Current tested compatible `Vite/Rollup/unplugin` plugins as below:

## Vite/Rollup Plugins
using `vitePlugins` in `farm.config.ts` to configure `Vite/Rollup` plugins.

```ts
import { UserConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import vueJsx from '@vitejs/plugin-vue-jsx';

const config: UserConfig = {
  vitePlugins: [
    vue(),
    vueJsx(),
  ]
}
```

* **[`@vitejs/plugin-vue`](https://github.com/vitejs/vite-plugin-vue/blob/main/packages/plugin-vue/README.md)**: Vue support.
* **[`@vitejs/plugin-vue-jsx`](https://github.com/vitejs/vite-plugin-vue/tree/main/packages/plugin-vue-jsx)**: Vue Jsx/Tsx support.
* **[`vite-plugin-solid`](https://www.npmjs.com/package/vite-plugin-solid)**: Solid support for Farm.
* **[`vite-plugin-mock`](https://www.npmjs.com/package/vite-plugin-solid)**: Mock data in both dev and prod.
* ...

## unplugin
:::note
Currently you can use `unplugin/vite` or `unplugin/rollup` in Farm. `unplugin/farm` will be available when [this PR](https://github.com/unjs/unplugin/pull/341) are merged into unplugin
:::

```ts
import Icons from 'unplugin-icons/vite';
import IconsResolver from 'unplugin-icons/resolver';
import Components from 'unplugin-vue-components/rollup';
import { NaiveUiResolver } from 'unplugin-vue-components/resolvers';
import { FileSystemIconLoader } from 'unplugin-icons/loaders';

const config: UserConfig = {
  vitePlugins: [
    Icons({
      compiler: 'vue3',
      customCollections: {
        [collectionName]: FileSystemIconLoader(localIconPath, svg =>
          svg.replace(/^<svg\s/, '<svg width="1em" height="1em" ')
        )
      },
      scale: 1,
      defaultClass: 'inline-block'
    }),
    Components({
      dts: 'src/typings/components.d.ts',
      types: [{ from: 'vue-router', names: ['RouterLink', 'RouterView'] }],
      resolvers: [
        NaiveUiResolver(),
        IconsResolver({ customCollections: [collectionName], componentPrefix: VITE_ICON_PREFIX })
      ]
    })
  ]
}
```

All unplugin are supported in Farm:
- [unplugin-auto-import](https://github.com/antfu/unplugin-auto-import)
- [unplugin-vue2-script-setup](https://github.com/antfu/unplugin-vue2-script-setup)
- [unplugin-icons](https://github.com/antfu/unplugin-icons)
- [unplugin-vue-components](https://github.com/antfu/unplugin-vue-components)
- [unplugin-upload-cdn](https://github.com/zenotsai/unplugin-upload-cdn)
- [unplugin-web-ext](https://github.com/jwr12135/unplugin-web-ext)
- [unplugin-properties](https://github.com/pd4d10/unplugin-properties)
- [unplugin-moment-to-dayjs](https://github.com/1247748612/unplugin-moment-to-dayjs)
- [unplugin-object-3d](https://github.com/m0ksem/unplugin-object-3d)
- [unplugin-parcel-css](https://github.com/ssssota/unplugin-parcel-css)
- [unplugin-vue](https://github.com/sxzz/unplugin-vue)
- [unplugin-vue-macros](https://github.com/sxzz/unplugin-vue-macros)
- [unplugin-vue-define-options](https://github.com/sxzz/unplugin-vue-macros/tree/main/packages/define-options)
- [unplugin-jsx-string](https://github.com/sxzz/unplugin-jsx-string)
- [unplugin-ast](https://github.com/sxzz/unplugin-ast)
- [unplugin-element-plus](https://github.com/element-plus/unplugin-element-plus)
- [unplugin-glob](https://github.com/sxzz/unplugin-glob)
- [unplugin-sentry](https://github.com/kricsleo/unplugin-sentry)
- [unplugin-imagemin](https://github.com/ErKeLost/unplugin-imagemin)
- [unplugin-typedotenv](https://github.com/ssssota/typedotenv)
- [unplugin-fonts](https://github.com/cssninjaStudio/unplugin-fonts)
- [sentry-javascript-bundler-plugins](https://github.com/getsentry/sentry-javascript-bundler-plugins)
- [unplugin-svg-component](https://github.com/Jevon617/unplugin-svg-component)
- [unplugin-vue-cssvars](https://github.com/baiwusanyu-c/unplugin-vue-cssvars)