import type { UserConfig } from 'farm';
import farmJsPluginVue from '@farmfe/js-plugin-vue';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    },
    resolve: {
      strictExports: true
    }
  },
  server: {
    port: 7009,
    strictPort: false,
  },
  plugins: [farmJsPluginVue()]
});
