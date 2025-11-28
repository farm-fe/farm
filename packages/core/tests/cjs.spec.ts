import path from 'path';
import { fileURLToPath } from 'url';
import { describe, expect, test } from 'vitest';

// @ts-ignore ignore error for cjs
import { normalizeDevServerConfig, resolveConfig } from '../dist/cjs/index.cjs';
import { isDisableCache } from '../src/config/env.js';

describe('resolveUserConfig', () => {
  test('resolveUserConfig', async () => {
    const filePath = fileURLToPath(path.dirname(import.meta.url));

    const config = await resolveConfig(
      {
        configFile: path.join(filePath, 'fixtures', 'config', 'farm.config.ts')
      },
      'dev',
      'development'
    );

    expect(config.compilation.define).toMatchSnapshot();
    expect(config.compilation.input).toEqual({
      main: './main.tsx'
    });
    expect(config.compilation.output).toEqual({
      clean: true,
      path: './dist',
      publicPath: '/',
      externalGlobals: {},
      targetEnv: 'browser'
    });
    expect(config.compilation.lazyCompilation).toEqual(true);
    expect(config.compilation.sourcemap).toEqual(true);
    expect(config.compilation.minify).toEqual(false);
    expect(config.compilation.presetEnv).toEqual(false);
    expect(config.server).toEqual(normalizeDevServerConfig(config));
  });

  test('resolveUserConfig with process.env.FARM_DISABLE_CACHE', async () => {
    const filePath = fileURLToPath(path.dirname(import.meta.url));

    for (const item of ['true', '']) {
      process.env.FARM_DISABLE_CACHE = item.toString();

      const config = await resolveConfig(
        {
          configFile: path.join(
            filePath,
            'fixtures',
            'config',
            'farm.config.ts'
          ),
          server: {
            hmr: false
          }
        },
        'dev',
        'development'
      );

      if (isDisableCache()) {
        expect(config.compilation.persistentCache).toEqual(false);
      } else {
        expect(config.compilation.persistentCache.cacheDir).toBeTruthy();
        // cache dir is related to the work directory, it should be omitted
        delete config.compilation.persistentCache.cacheDir;

        expect(config.compilation.persistentCache).toEqual({
          buildDependencies: [
            path.join(
              process.cwd(),
              '@farmfe',
              'runtime',
              'src',
              'modules',
              'module-helper.ts.farm-runtime'
            ),
            path.join(filePath, 'fixtures', 'config', 'farm.config.ts'),
            path.join(filePath, 'fixtures', 'config', 'util.ts'),
            'module',
            'package-lock.json',
            'pnpm-lock.yaml',
            'yarn.lock'
          ],
          envs: {
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.BASE_URL': '"/"',
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.DEV': 'true',
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.PROD': 'false',
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.NODE_ENV':
              JSON.stringify('development'),
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.mode':
              JSON.stringify('development'),
            '$__farm_regex:(global(This)?\\.)?process\\.env\\.FARM_DISABLE_CACHE':
              JSON.stringify(item),
            NODE_ENV: 'development',
            FARM_PROCESS_ENV:
              '{"FARM_DISABLE_CACHE":"","NODE_ENV":"development","BASE_URL":"/","mode":"development","DEV":true,"PROD":false}',
            FARM_DISABLE_CACHE: item,
            mode: 'development',
            BASE_URL: '/',
            DEV: true,
            PROD: false,
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
