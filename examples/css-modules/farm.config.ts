import type { UserConfig } from '@farmfe/core';
import postcss from '@farmfe/js-plugin-postcss';

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
    // record: true,
    sourcemap: true
    // treeShaking: true,
    // minify: true,
  },
  server: {
    open: true
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass', postcss()]
});
