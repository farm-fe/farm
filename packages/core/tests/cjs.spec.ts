import path from 'path';
import { fileURLToPath } from 'url';
import { expect, test } from 'vitest';

import { normalizeDevServerOptions, resolveConfig } from '../src/index.js';
import { Logger } from '../src/utils/logger.js';

test('resolveUserConfig', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  const config = await resolveConfig(
    { configPath: path.join(filePath, 'fixtures', 'config', 'farm.config.ts') },
    new Logger(),
    'development'
  );
  console.log(config.compilation.define);

  expect(config.compilation.define).toEqual({
    // FARM_HMR_HOST: true,
    // FARM_HMR_PATH: '/__hmr',
    // FARM_HMR_PORT: '9000',
    FARM_PROCESS_ENV: {
      NODE_ENV: 'development'
    },
    // FARM_HMR_PROTOCOL: 'ws',
    '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV': '"development"'
  });
  expect(config.compilation.input).toEqual({
    main: './main.tsx'
  });
  expect(config.compilation.output).toEqual({
    path: './dist',
    publicPath: '/',
    targetEnv: 'browser'
  });
  expect(config.compilation.lazyCompilation).toEqual(true);
  expect(config.compilation.sourcemap).toEqual(true);
  expect(config.compilation.minify).toEqual(false);
  expect(config.compilation.presetEnv).toEqual(false);
  expect(config.compilation.persistentCache).toEqual({
    buildDependencies: [
      // path.join(filePath, '..', 'src', 'config.ts'),
      path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
      path.join(filePath, 'fixtures', 'config', 'util.ts'),
      'module',
      'package-lock.json',
      'pnpm-lock.yaml',
      'yarn.lock'
    ],
    envs: {
      FARM_PROCESS_ENV: '{"NODE_ENV":"development"}',
      NODE_ENV: 'development',
      'package.json[name]': 'farm-fe',
      'package.json[type]': 'unknown',
      '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV':
        '"development"',
      'package.json[browser]': 'unknown',
      'package.json[exports]': 'unknown',
      'package.json[main]': 'unknown',
      'package.json[module]': 'unknown'
      // FARM_HMR_HOST: 'true',
      // FARM_HMR_PATH: '/__hmr',
      // FARM_HMR_PORT: '9000',
      // FARM_HMR_PROTOCOL: 'ws'
    },
    moduleCacheKeyStrategy: {}
  });
  expect(config.server).toEqual(
    normalizeDevServerOptions(config.server, 'development')
  );
});
