import type { UserConfig } from '@farmfe/core';
// import vue from '@farmfe/js-plugin-vue';
import vue from '@vitejs/plugin-vue';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  // plugins: [vue()],
  vitePlugins: [() => ({ vitePlugin: vue(), filters: ['.vue$', '.*plugin-vue:export-helper'] })]
});
