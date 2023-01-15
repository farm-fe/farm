import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['**/*.spec.ts'],
    coverage: {
      reporter: ['json'],
    },
    environment: 'node',
    deps: {
      interopDefault: false,
    },
  },
});
