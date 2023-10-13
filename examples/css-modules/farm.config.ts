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
    },
    sourcemap: true
    // treeShaking: true,
    // minify: true,
  },
  server: {
    hmr: {
      port: 9802,
    },
    port: 9001,
    open: true,
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass'],
});
