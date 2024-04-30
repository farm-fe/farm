import { defineConfig } from '@farmfe/core';
import Pages from 'vite-plugin-pages';

import farmPluginRemoveConsole from 'farm-plugin-remove-console';

export default defineConfig({
  plugins: [
    '@farmfe/plugin-react',
    farmPluginRemoveConsole({
      include: ['src/**/*']
    })
  ],
  vitePlugins: [
    Pages({
      resolver: 'react'
    })
  ]
});
