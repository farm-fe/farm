import { defineConfig } from '@farmfe/core';
import { resolve } from 'path';
export default defineConfig({
  // compilation: {
  //   mode: 'staging'
  // },
  compilation: {
    persistentCache: false,
  },
  envPrefix: ['FARM_', 'CUSTOM_PREFIX_', 'NEW_'],
  envDir: resolve('./env'),
  server: {
    port: 7667
  }
});
