import { defineConfig } from 'farm';
import NestPlugin from './index.plugin';

export default defineConfig({
  plugins: [NestPlugin()],
});
