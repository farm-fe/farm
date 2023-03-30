import { defineConfig } from 'tsup';
export default defineConfig({
  minify: true,
  bundle: true,
  entry: ['./index.ts'],
  platform: 'node',
  target: 'node16',
  treeshake: true,
});
