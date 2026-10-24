import { defineConfig } from '@farmfe/core';
import jsx from '@vitejs/plugin-vue-jsx';
import { vueCommon } from './configs/vue-common';

const config = vueCommon('js');
config.compilation!.input = { index: './apps/vue-jsx-lab/index.html' };
config.compilation!.output = { path: 'dist/vue-jsx-js' };
config.compilation!.persistentCache = { cacheDir: `.cache/vue-jsx-js/${process.env.COMMERCE_CACHE_ID ?? 'default'}` };
export default defineConfig({ ...config, vitePlugins: [jsx()] });
