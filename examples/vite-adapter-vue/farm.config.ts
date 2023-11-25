import type { UserConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import AutoImport from 'unplugin-auto-import/vite';
import Components from 'unplugin-vue-components/vite';
import { ElementPlusResolver } from 'unplugin-vue-components/resolvers';
import farmJsPluginSass from '@farmfe/js-plugin-sass';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    presetEnv: false
  },
  plugins: [
    // farmJsPluginSass()
    '@farmfe/plugin-sass',
    {
      name: 'remove-css-filter-plugin',
      priority: 0,
      transform: {
        filters: {
          resolvedPaths: [
            'src/components/HelloWorld.vue\\?vue&(.+)&lang\\.scss'
          ]
        },
        executor({ content }) {
          return {
            content: content.replace('filter: alpha(opacity=0);', '')
          };
        }
      }
    }
  ],
  vitePlugins: [
    {
      name: 'vite111',
      config(config, env) {
        console.log(config, env);
      }
    },
    vue(),
    AutoImport({
      resolvers: [ElementPlusResolver({ importStyle: 'sass' })]
    }),
    Components({
      resolvers: [ElementPlusResolver({ importStyle: 'sass' })]
    })
  ]
});
