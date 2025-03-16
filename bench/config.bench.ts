import { describe, bench } from 'vitest';
import { resolveConfig } from '@farmfe/core';
import { getExampleRoot } from './utils';

describe('resolve config', () => {
  describe('react', () => {
    const root = getExampleRoot('react');
    bench('resolve react example config', async () => {
      await resolveConfig({ root }, 'build');
    });
  });

  describe('vue3', () => {
    const root = getExampleRoot('vue3');
    bench('resolve vue3 example config', async () => {
      await resolveConfig({ root }, 'build');
    });
  });
});
