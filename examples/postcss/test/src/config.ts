import type { UserConfig } from '@farmfe/core';
import farmPostcssPlugin from '@farmfe/js-plugin-postcss';
import path from 'path';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  plugins: [
    farmPostcssPlugin({
      cwd: path.resolve(process.cwd(), './test')
    })
  ],
  compilation: {
    root: path.resolve(process.cwd(), './test')
  }
});
