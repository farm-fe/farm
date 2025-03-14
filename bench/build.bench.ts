import { bench, describe } from 'vitest';
import { build } from '@farmfe/core';
import { getExampleRoot } from './utils';

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
