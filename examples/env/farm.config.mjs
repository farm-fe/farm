// import type { UserConfig } from '@farmfe/core';
import { resolve } from 'path';
function defineConfig(config) {
  return config;
}

export default defineConfig({
  // compilation: {
  //   mode: 'staging'
  // },
  envPrefix: ['FARM_', 'CUSTOM_PREFIX_', 'NEW_'],
  envDir: resolve(process.cwd(), './env'),
  server: {
    port: 7667
  }
});
