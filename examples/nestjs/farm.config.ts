import { defineConfig } from '@farmfe/core';
import NestPlugin from './index.plugin.js';

export default defineConfig({
  plugins: [NestPlugin()]
});
