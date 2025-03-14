import { describe, bench } from 'vitest';
import { resolveConfig } from '@farmfe/core';
import { getExampleRoot } from './utils';

describe('resolve config', () => {
  describe('react', () => {
    bench('build react example', async () => {
      await resolveConfig({ root: getExampleRoot('react') }, 'build');
    });
  });

  describe('vue3', () => {
    bench('build vue3 example', async () => {
      await resolveConfig({ root: getExampleRoot('vue3') }, 'build');
    });
  });
});
