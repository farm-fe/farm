import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import { vueCommon } from './configs/vue-common';

export default defineConfig({ ...vueCommon('js'), vitePlugins: [vue()] });
