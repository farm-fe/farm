import { defineConfig } from '@farmfe/core';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  vitePlugins: [svelte()],
  compilation: {
    resolve: {
      // mainFields: ['exports', 'main'],
    }
  }
});
