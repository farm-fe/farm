import type { UserConfig } from '@farmfe/core';
import less from '@farmfe/js-plugin-less';

function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    resolve: {
      alias: {
        'stream$': 'readable-stream'
      }
    },
    presetEnv: false,
  },
  server: {
    port: 6699,
  },
  plugins: [
    less({}),
  ]
});
