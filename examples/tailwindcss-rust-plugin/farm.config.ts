import { defineConfig } from '@farmfe/core';
import tailwindConfig from './tailwindcss.config.js';

export default defineConfig({
  compilation: {
    input: {
      index: './index.html'
    },
    persistentCache: false,
    progress: false
  },
  plugins: [
    '@farmfe/plugin-react',
    ['@farmfe/plugin-tailwindcss', { config: tailwindConfig }]
  ]
});
