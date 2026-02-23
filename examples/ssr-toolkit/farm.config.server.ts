import { defineConfig } from '@farmfe/core';
import farmLessPlugin from '@farmfe/js-plugin-less';
import vue from '@vitejs/plugin-vue';


export default defineConfig({
  plugins: [farmLessPlugin()],
  vitePlugins: [vue()],
  compilation: {
    input: {
      index: './src/entry-server.ts'
    },
    output: {
      path: 'dist/server',
      targetEnv: 'node',
      format: 'esm'
    },
    minify: false
  },
  server: {
    middlewareMode: true
  }
});
