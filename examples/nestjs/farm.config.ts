import { defineConfig } from 'farm';
import NestPlugin from './index.plugin.js';

export default defineConfig({
  plugins: [NestPlugin()]
});
