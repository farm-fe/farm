import codspeedPlugin from '@codspeed/vitest-plugin';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [codspeedPlugin()],
  test: {
    include: ['packages/**/*.spec.ts', 'js-plugins/**/*.spec.ts'],
    coverage: {
      reporter: ['json']
    },
    pool: 'forks',
    environment: 'node',
    deps: {
      interopDefault: false
    },
    retry: 5
  }
});
