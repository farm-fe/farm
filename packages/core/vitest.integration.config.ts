import { defineConfig } from 'vitest/config';

export default defineConfig({
  test: {
    include: ['packages/core/tests/issue-native.integration.ts'],
    maxWorkers: 1,
    minWorkers: 1,
    pool: 'forks'
  }
});
