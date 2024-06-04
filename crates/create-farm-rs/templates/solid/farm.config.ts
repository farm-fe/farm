import { defineConfig } from '@farmfe/core';
import solid from 'vite-plugin-solid';

export default defineConfig({
  vitePlugins: [
    () => ({
      vitePlugin: solid(),
      filters: ['\\.tsx$', '\\.jsx$']
    })
  ]
});
