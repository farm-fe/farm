import { defineConfig } from '@farmfe/core';
import preact from '@preact/preset-vite';

export default defineConfig({
  vitePlugins: [preact()]
});
