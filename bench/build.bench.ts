import { bench, describe } from 'vitest';
import { build } from '@farmfe/core';
import { getFixtureRoot } from './utils';

describe('build fixture', { sequential: true }, () => {
  describe('vanilla', () => {
    const root = getFixtureRoot('vanilla');
    bench('build vanilla fixture', async () => {
      await build({ root });
    });
  });
});
