import type { UserConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  vitePlugins: [vue()]
});
