import { readdirSync, statSync } from 'node:fs';
import { join } from 'node:path';
import { defineConfig } from 'vitest/config';

function resolveE2eIncludePatterns() {
  const selectedExample = process.env.FARM_E2E_EXAMPLE;
  const startFromExample = process.env.FARM_EXAMPLE_START_FROM;

  const examples = readdirSync('./examples')
    .filter((name) => statSync(join('./examples', name)).isDirectory())
    .sort((a, b) => a.localeCompare(b));

  if (selectedExample) {
    if (!examples.includes(selectedExample)) {
      throw new Error(
        `FARM_E2E_EXAMPLE '${selectedExample}' was not found under ./examples`
      );
    }

    return ['e2e/**/*.spec.ts', `examples/${selectedExample}/**/*.spec.ts`];
  }

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

/**
 * LEGACY: This file is no longer used.
 *
 * The new standalone E2E runner (`scripts/test-e2e.mts`) no longer uses vitest.
 *
 * Key changes:
 *   - No vitest config needed
 *   - No global setup/teardown files
 *   - Test discovery: examples/*/e2e.spec.ts (or default smoke test if no spec exists)
 *   - Test execution: SpecRunner in e2e/runner.ts
 *   - Browser: Launched directly by scripts/test-e2e.mts
 *
 * See scripts/test-e2e.mts for the new implementation.
 */
