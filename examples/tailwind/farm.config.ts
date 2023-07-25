import type { UserConfig } from '@farmfe/core';
import farmPostcssPlugin from '@farmfe/js-plugin-postcss';

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
    sourcemap: true
  },
  server: {
    hmr: true
  },
  plugins: ['@farmfe/plugin-react', farmPostcssPlugin()]
});
