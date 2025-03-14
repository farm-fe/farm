import path from 'node:path';
import { bench, describe } from 'vitest';
import { build } from './index.js';

const cwd = process.cwd();

function getExampleRoot(name: string) {
  return path.join(cwd, 'examples', name);
}

describe('build example', { sequential: true }, () => {
  describe('react', () => {
    bench('build react example', async () => {
      await build({ root: getExampleRoot('react') });
    });
  });

  describe('vue3', () => {
    bench('build vue3 example', async () => {
      await build({ root: getExampleRoot('vue3') });
    });
  });
});
