import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    output: {
      path: './build'
    },
    resolve: {
      strictExports: true
    }
  },
  server: {
    proxy: {
      '^/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      }
    }
  }
});
