import path from 'path';
import { pathToFileURL } from 'url';
import { expect, test } from 'vitest';
import { Compiler, normalizeUserCompilationConfig } from '../src/index.js';
import { JsPlugin } from '../src/plugin/index.js';

function getJsPluginsFixturesDir() {
  const currentDir = path.dirname(new URL(import.meta.url).pathname);
  return path.resolve(currentDir, 'fixtures', 'js-plugins');
}

function getOutputFilePath() {
  const root = getJsPluginsFixturesDir();
  return path.join(root, 'dist', 'index.mjs');
}

async function getCompiler(plugins: JsPlugin[]): Promise<Compiler> {
  const root = getJsPluginsFixturesDir();
  const config = await normalizeUserCompilationConfig(
    {
      root,
      compilation: {
        input: {
          index: './index.ts',
        },
        output: {
          filename: 'index.mjs',
        },
        lazyCompilation: false,
      },
      server: {
        hmr: false,
      },
      plugins,
    },
    'production'
  );
  return new Compiler(config);
}

test('Js Plugin Execution - resolve', async () => {
  const root = getJsPluginsFixturesDir();
  const resolvedPath = path.join(root, 'resolved.ts');
  const compiler = await getCompiler([
    {
      name: 'test-resolve',
      priority: 1000,
      resolve: {
        filters: {
          sources: ['\\./index.ts'],
          importers: ['None'],
        },
        executor: async (param) => {
          console.log(param);

          return {
            resolvedPath,
            query: {},
            sideEffects: false,
            external: false,
          };
        },
      },
    },
  ]);

  await compiler.compile();
  const outputFilePath = getOutputFilePath();

  if (process.platform === 'win32') {
    const result = await import(pathToFileURL(outputFilePath).toString());
    expect(result.a).toBe(2);
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe(2);
  }
});
