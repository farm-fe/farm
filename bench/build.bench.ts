import { bench, describe } from 'vitest';
import { build } from '@farmfe/core';
import { getExampleRoot } from './utils';

describe('build example', { sequential: true }, () => {
  describe('react', () => {
    const root = getExampleRoot('react');
    bench('build react example', async () => {
      await build({ root });
    });
  });

  describe('vue3', () => {
    const root = getExampleRoot('vue3');
    bench('build vue3 example', async () => {
      await build({ root });
    });
  });
});
