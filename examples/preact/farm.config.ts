import { defineConfig } from 'farm';
import preact from '@preact/preset-vite';
import react from '@farmfe/plugin-react';

export default defineConfig({
  plugins: [react()],
  vitePlugins: [preact()]
});
