import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { defineConfig } from 'vitest/config';

const directory = path.dirname(fileURLToPath(import.meta.url));
export default defineConfig({
  resolve: {
    alias: {
      '@farmfe/core': path.resolve(
        directory,
        '../../packages/core/src/index.ts'
      )
    }
  },
  test: {
    include: ['js-plugins/postcss/src/**/*.spec.ts'],
    maxWorkers: 1,
    minWorkers: 1
  }
});
