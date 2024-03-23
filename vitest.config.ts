import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['packages/**/*.spec.ts', "js-plugins/**/*.spec.ts"],
    coverage: {
      reporter: ['json'],
    },
    environment: 'node',
    deps: {
      interopDefault: false,
    },
  },
});
