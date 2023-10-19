import type { UserConfig } from '@farmfe/core';
import farmPostcssPlugin from '@farmfe/js-plugin-postcss';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  plugins: [farmPostcssPlugin() ],
});
