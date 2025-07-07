import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    setupFiles: ['./e2e/vitestSetup.ts'],
    globalSetup: ['./e2e/vitestGlobalSetup.ts'],
    include: ['examples/**/*.spec.ts', `e2e/**/*.spec.ts`],
    hookTimeout: 600_000,
    testTimeout: 600_000,
    isolate: false,
    sequence: {
      concurrent: false
    },
    poolOptions: {
      threads: {
        singleThread: true
      }
    }
  }
});
