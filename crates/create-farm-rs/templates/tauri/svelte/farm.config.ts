import { defineConfig } from 'farm';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  vitePlugins: [svelte()],
  server: {
    port: 1420
  }
});
