/**
 * LEGACY: This file is no longer used.
 *
 * The new standalone E2E runner is: scripts/test-e2e.mts
 *
 * This was the old vitest-based orchestrator.  All functionality has been
 * ported to the new runner, which:
 *   - Uses subprocess + Playwright (not vitest)
 *   - Supports --example, --from, --start-from, --project flags
 *   - Runs custom spec files (e2e.spec.ts) or default smoke tests
 *   - Manages browser lifecycle directly
 *
 * To run E2E tests:
 *   npm run test-e2e                 # Run all examples
 *   npm run test-e2e -- --example react    # Single example
 *   npm run test-e2e -- --from arco-pro    # From that example onward
 *
 * See scripts/test-e2e.mts for implementation details.
 */
import { execa } from 'execa';

function parseE2eArgs(argv) {
  let example;
  let startFrom;
  const passThroughArgs = [];

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];

    if (arg === '--example' || arg === '--project') {
      example = argv[i + 1];
      i += 1;
      continue;
    }

    if (arg.startsWith('--example=')) {
      example = arg.slice('--example='.length);
      continue;
    }

    if (arg.startsWith('--project=')) {
      example = arg.slice('--project='.length);
      continue;
    }

    if (arg === '--from' || arg === '--start-from') {
      startFrom = argv[i + 1];
      i += 1;
      continue;
    }

    if (arg.startsWith('--from=')) {
      startFrom = arg.slice('--from='.length);
      continue;
    }

    if (arg.startsWith('--start-from=')) {
      startFrom = arg.slice('--start-from='.length);
      continue;
    }

    passThroughArgs.push(arg);
  }

  return { example, startFrom, passThroughArgs };
}

const { example, startFrom, passThroughArgs } = parseE2eArgs(
  process.argv.slice(2)
);

if (example) {
  console.log(`Running e2e for example: ${example}`);
}

if (startFrom) {
  console.log(`Running e2e from example: ${startFrom}`);
}

if (example && startFrom) {
  console.log(
    `Both --example and --from were provided. --example takes precedence, ignoring --from.`
  );
}

await execa(
  'pnpm',
  ['exec', 'vitest', 'run', '-c', 'vitest.config.e2e.ts', ...passThroughArgs],
  {
    stdio: 'inherit',
    env: {
      ...process.env,
      ...(example ? { FARM_E2E_EXAMPLE: example } : {}),
      ...(startFrom ? { FARM_EXAMPLE_START_FROM: startFrom } : {})
    }
  }
);