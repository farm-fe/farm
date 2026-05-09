import { readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { defineConfig } from 'vitest/config';

function resolveE2eIncludePatterns() {
  const startFromExample = process.env.FARM_EXAMPLE_START_FROM;

  const examples = readdirSync('./examples')
    .filter((name) => statSync(join('./examples', name)).isDirectory())
    .sort((a, b) => a.localeCompare(b));
  const startIndex = startFromExample ? examples.indexOf(startFromExample) : 0;

  if (startFromExample && startIndex === -1) {
    throw new Error(
      `FARM_EXAMPLE_START_FROM '${startFromExample}' was not found under ./examples`
    );
  }

  const selectedExamples = examples.slice(startIndex);

  return [
    'e2e/**/*.spec.ts',
    ...selectedExamples.map((name) => `examples/${name}/**/*.spec.ts`)
  ];
}

export default defineConfig({
  test: {
    setupFiles: ['./e2e/vitestSetup.ts'],
    globalSetup: ['./e2e/vitestGlobalSetup.ts'],
    include: resolveE2eIncludePatterns(),
    hookTimeout: 600_000,
    testTimeout: 600_000,
    isolate: false,
    sequence: {
      concurrent: false
    },
    maxConcurrency: 2,
    poolOptions: {
      threads: {
        singleThread: true
      }
    }
  }
});
