import path from 'node:path';
import { build } from '@farmfe/core';
import { bench, describe } from 'vitest';

async function build_react_example() {
  await build({
    root: path.resolve(process.cwd(), './examples/react'),
    configPath: path.resolve('./examples/react'),
    compilation: { input: {}, output: {} }
  });
}

describe('react_example_bench', () => {
  let hasRun = false;
  bench(
    'build_react_example',
    async () => {
      if (!hasRun) {
        await build_react_example();
        hasRun = true;
      }
    },
    { warmupIterations: 0 }
  );
});
