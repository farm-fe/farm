import { defineConfig } from 'farm';
import preact from '@preact/preset-vite';

export default defineConfig({
  vitePlugins: [preact()]
});
