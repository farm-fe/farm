import type { UserConfig } from '@farmfe/core';
import { resolve } from 'path';
function defineConfig(config: UserConfig) {
  return config;
}

export default defineConfig({
  compilation: {
    mode: 'staging'
  },
  envPrefix: ['FARM_', 'CUSTOM_PREFIX_'],
  envDir: resolve(process.cwd(), './env')
});
