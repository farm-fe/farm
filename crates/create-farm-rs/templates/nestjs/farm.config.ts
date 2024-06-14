import { defineConfig } from '@farmfe/core';
import NestPlugin from './index.plugin.ts';

export default defineConfig({
  plugins: [NestPlugin()],
});
