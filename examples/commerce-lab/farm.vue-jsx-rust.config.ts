import { defineConfig } from '@farmfe/core';
import { vueCommon } from './configs/vue-common';

const config = vueCommon('rust');
config.compilation!.input = { index: './apps/vue-jsx-lab/index.html' };
config.compilation!.output = { path: 'dist/vue-jsx-rust' };
config.compilation!.persistentCache = { cacheDir: `.cache/vue-jsx-rust/${process.env.COMMERCE_CACHE_ID ?? 'default'}` };
export default defineConfig({ ...config, plugins: ['@farmfe/plugin-vue-jsx'] });
