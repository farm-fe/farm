import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { expect, test } from 'vitest';
import { Compiler, normalizeUserCompilationConfig } from '../src/index.js';
import { JsPlugin } from '../src/plugin/index.js';

function getJsPluginsFixturesDir() {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(currentDir, 'fixtures', 'js-plugins');
}

function getOutputFilePath(p: string) {
  const root = getJsPluginsFixturesDir();
  return path.join(root, 'dist', p, 'index.mjs');
}

async function getCompiler(p: string, plugins: JsPlugin[]): Promise<Compiler> {
  const root = getJsPluginsFixturesDir();
  const config = await normalizeUserCompilationConfig(
    {
      root,
      compilation: {
        input: {
          index: './index.ts',
        },
        output: {
          path: path.join('dist', p),
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
  const compiler = await getCompiler('resolve', [
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
  await compiler.writeResourcesToDisk();
  const outputFilePath = getOutputFilePath('resolve');

  if (process.platform === 'win32') {
    const result = await import(pathToFileURL(outputFilePath).toString());
    expect(result.a).toBe(2);
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe(2);
  }
});

test('Js Plugin Execution - load', async () => {
  const root = getJsPluginsFixturesDir();
  const compiler = await getCompiler('load', [
    {
      name: 'test-load',
      priority: 1000,
      load: {
        filters: {
          resolvedPaths: [path.join(root, 'index.ts').replaceAll('\\', '\\\\')],
        },
        executor: async (param) => {
          console.log(param);
          return {
            content: 'export default 33;',
            moduleType: 'Ts',
          };
        },
      },
    },
  ]);

  await compiler.compile();
  await compiler.writeResourcesToDisk();
  const outputFilePath = getOutputFilePath('load');

  if (process.platform === 'win32') {
    const result = await import(pathToFileURL(outputFilePath).toString());
    expect(result.a).toBe(33);
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe(33);
  }
});

test('Js Plugin Execution - transform', async () => {
  const root = getJsPluginsFixturesDir();
  const compiler = await getCompiler('transform', [
    {
      name: 'test-transform',
      priority: 1000,
      transform: {
        filters: {
          resolvedPaths: [path.join(root, 'index.ts').replaceAll('\\', '\\\\')],
        },
        executor: async (param) => {
          console.log(param);
          return {
            content: 'export default 44;',
          };
        },
      },
    },
  ]);

  await compiler.compile();
  await compiler.writeResourcesToDisk();
  const outputFilePath = getOutputFilePath('transform');

  if (process.platform === 'win32') {
    const result = await import(pathToFileURL(outputFilePath).toString());
    expect(result.a).toBe(44);
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe(44);
  }
});
