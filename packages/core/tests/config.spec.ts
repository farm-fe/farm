import path from 'path';
import { fileURLToPath } from 'url';
import { test, expect } from 'vitest';

import {
  DEFAULT_DEV_SERVER_OPTIONS,
  normalizeDevServerOptions,
  resolveUserConfig,
} from '../src/index.js';
import { DefaultLogger } from '../src/logger.js';

test('resolveUserConfig', async () => {
  const filePath = fileURLToPath(path.dirname(import.meta.url));

  console.log(
    '\n\n\n\n',
    filePath,
    path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
    '\n\n\n\n\n'
  );
  const config = await resolveUserConfig(
    path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
    new DefaultLogger()
  );

  expect(config).toEqual({
    compilation: {
      input: {
        main: './main.tsx',
      },
    },
    root: path.join(filePath, 'fixtures', 'config'),
  });
});

test('normalize-dev-server-options', () => {
  let options = normalizeDevServerOptions({});
  expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
  expect(options.port).toBe(DEFAULT_DEV_SERVER_OPTIONS.port);
  expect(options.writeToDisk).toBe(DEFAULT_DEV_SERVER_OPTIONS.writeToDisk);

  options = normalizeDevServerOptions({ writeToDisk: true });
  expect(options.https).toBe(DEFAULT_DEV_SERVER_OPTIONS.https);
  expect(options.port).toBe(DEFAULT_DEV_SERVER_OPTIONS.port);
  expect(options.writeToDisk).toBe(true);
});
