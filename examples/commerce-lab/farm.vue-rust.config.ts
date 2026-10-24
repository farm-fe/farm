import { defineConfig } from '@farmfe/core';
import { vueCommon } from './configs/vue-common';

export default defineConfig({ ...vueCommon('rust'), plugins: ['@farmfe/plugin-vue'] });
