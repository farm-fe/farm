import type { UserConfig } from '@farmfe/core';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    input: {
      index: './index.html',
    },
    output: {
      path: './build',
      publicPath: process.env.NODE_ENV === 'development' ? '/admin/' : '/',
    },
    sourcemap: true,
  },
  server: {
    hmr: true,
    writeToDisk: false,
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass'],
});
