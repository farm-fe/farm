import { execa } from 'execa';

function parseStartFromArg(argv) {
  let startFrom;
  const passThroughArgs = [];

  for (let i = 0; i < argv.length; i++) {
    const arg = argv[i];

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

  return { startFrom, passThroughArgs };
}

const { startFrom, passThroughArgs } = parseStartFromArg(process.argv.slice(2));

if (startFrom) {
  console.log(`Running e2e from example: ${startFrom}`);
}

await execa(
  'pnpm',
  ['exec', 'vitest', 'run', '-c', 'vitest.config.e2e.ts', ...passThroughArgs],
  {
    stdio: 'inherit',
    env: {
      ...process.env,
      ...(startFrom ? { FARM_EXAMPLE_START_FROM: startFrom } : {})
    }
  }
);