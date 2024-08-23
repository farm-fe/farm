import path from 'path';
import { fileURLToPath } from 'url';
import { describe, expect, test } from 'vitest';

import { isDisableCache } from '../src/config/env.js';
import { normalizeDevServerConfig, resolveConfig } from '../src/index.js';
import { Logger } from '../src/utils/logger.js';

describe('resolveUserConfig', () => {
  test('resolveUserConfig', async () => {
    const filePath = fileURLToPath(path.dirname(import.meta.url));

    const config = await resolveConfig(
      {
        configPath: path.join(filePath, 'fixtures', 'config', 'farm.config.ts')
      },
      'development',
      new Logger()
    );
    console.log(config.compilation.define);

    expect(config.compilation.define).toMatchSnapshot();
    expect(config.compilation.input).toEqual({
      main: './main.tsx'
    });
    expect(config.compilation.output).toEqual({
      clean: true,
      path: './dist',
      publicPath: '/',
      targetEnv: 'browser'
    });
    expect(config.compilation.lazyCompilation).toEqual(true);
    expect(config.compilation.sourcemap).toEqual(true);
    expect(config.compilation.minify).toEqual(false);
    expect(config.compilation.presetEnv).toEqual(false);
    expect(config.server).toEqual(
      normalizeDevServerConfig(config.server, 'development')
    );
  });

  test('resolveUserConfig with process.env.DISABLE_CACHE', async () => {
    const filePath = fileURLToPath(path.dirname(import.meta.url));

    for (const item of ['true', '']) {
      process.env.DISABLE_CACHE = item.toString();

      const config = await resolveConfig(
        {
          configPath: path.join(
            filePath,
            'fixtures',
            'config',
            'farm.config.ts'
          ),
          server: {
            hmr: false
          }
        },
        'development',
        new Logger()
      );
      if (isDisableCache()) {
        expect(config.compilation.persistentCache).toEqual(false);
      } else {
        expect(config.compilation.persistentCache).toEqual({
          buildDependencies: [
            path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
            path.join(filePath, 'fixtures', 'config', 'util.ts'),
            'module',
            'package-lock.json',
            'pnpm-lock.yaml',
            'yarn.lock'
          ],
          envs: {
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV':
              JSON.stringify('development'),
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.mode':
              JSON.stringify('development'),
            FARM_PROCESS_ENV: '{"NODE_ENV":"development","mode":"development"}',
            FARM_RUNTIME_TARGET_ENV: '"browser"',
            NODE_ENV: 'development',
            mode: 'development',
            'package.json[browser]': 'unknown',
            'package.json[exports]': 'unknown',
            'package.json[main]': 'unknown',
            'package.json[module]': 'unknown',
            'package.json[name]': 'farm-fe',
            'package.json[type]': 'unknown'
          },
          moduleCacheKeyStrategy: {}
        });
      }
    }
  });
});
