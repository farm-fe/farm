import type { UserConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import AutoImport from 'unplugin-auto-import/vite'
import Components from 'unplugin-vue-components/vite'
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers'
import farmJsPluginSass from '@farmfe/js-plugin-sass'

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    presetEnv: false
  },
  plugins: [
    farmJsPluginSass()
  ],
  vitePlugins: [
    vue(),
    // ...
    AutoImport({
      resolvers: [ElementPlusResolver()],
    }),
    Components({
      resolvers: [ElementPlusResolver()],
    }),
  ]
});
