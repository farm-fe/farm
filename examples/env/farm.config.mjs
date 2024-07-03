import { defineConfig } from '@farmfe/core';
export default defineConfig({
  compilation: {
    persistentCache: false,
  },
  envPrefix: ['FARM_', 'CUSTOM_PREFIX_', 'NEW_'],
});
