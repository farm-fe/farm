import { describe, bench } from 'vitest';
import { resolveConfig } from '@farmfe/core';
import { getFixtureRoot } from './utils';

describe('resolve config', () => {
  describe('vanilla', () => {
    const root = getFixtureRoot('vanilla');
    bench('resolve vanilla fixture config', async () => {
      await resolveConfig({ root }, 'build');
    });
  });
});
