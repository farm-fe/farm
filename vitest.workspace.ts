import { defineWorkspace } from 'vitest/config';

export default defineWorkspace([
  './vitest.config.e2e.ts',
  './vitest.config.ts',
  './examples/vite-adapter-vue-sfc-src/vite.config.ts'
]);
