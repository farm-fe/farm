import { defineConfig } from 'farm';
import solid from 'vite-plugin-solid';

export default defineConfig({
  vitePlugins: [
    () => ({
      vitePlugin: solid(),
      filters: ['\\.tsx$', '\\.jsx$']
    })
  ],
  server: {
    port: 1420
  }
});
