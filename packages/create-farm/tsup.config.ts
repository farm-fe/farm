import { defineConfig } from 'tsup';
export default defineConfig({
  // splitting: false,
  // sourcemap: true,
  // clean: true
  bundle: true,
  entry: ['./index.ts'],
  watch: true,
  platform: 'node',
  target: 'node16',
  // treeShaking: true,
});
