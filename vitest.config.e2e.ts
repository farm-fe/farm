import { resolve } from 'node:path';
import { defineConfig } from 'vitest/config'


export default defineConfig({
  resolve: {
    alias: {
      '~utils': resolve(__dirname, './examples/test-utils'),
    }
  },
  test: {
    setupFiles: ['./examples/vitestSetup.ts'],
    globalSetup: ['./examples/vitestGlobalSetup.ts'],
    include: ['examples/**/*.spec.ts'],
    hookTimeout: 50_000
  }
});
