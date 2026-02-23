import { defineConfig } from '@farmfe/core';
import farmLessPlugin from '@farmfe/js-plugin-less';
import vue from '@vitejs/plugin-vue';

export default defineConfig({
  plugins: [farmLessPlugin()],
  vitePlugins: [vue()],
  server: {
    middlewareMode: true,
    preview: {
      distDir: 'dist/client'
    }
  },
  compilation: {
    minify: false,
    input: {
      index: './index.html'
    },
    output: {
      path: 'dist/client'
    }
  }
});
