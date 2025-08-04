import { defineConfig } from 'farm'
import { svelte } from '@sveltejs/vite-plugin-svelte'

// https://vitejs.dev/config/
export default defineConfig({
  vitePlugins: [svelte()],
  compilation: {
    persistentCache: false
  }
})
