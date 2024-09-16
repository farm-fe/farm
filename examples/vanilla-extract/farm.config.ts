import { defineConfig } from '@farmfe/core';
import { vanillaExtractPlugin } from '@vanilla-extract/vite-plugin';

export default defineConfig({
  vitePlugins: [() => ({
    vitePlugin: vanillaExtractPlugin(),
    filters: ['.+']
  })],
  compilation: {
    presetEnv: false
  }
});
