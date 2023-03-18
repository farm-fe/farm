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

async function getCompiler(
  p: string,
  plugins: JsPlugin[],
  input?: Record<string, string>
): Promise<Compiler> {
  const root = getJsPluginsFixturesDir();
  const config = await normalizeUserCompilationConfig(
    {
      root,
      compilation: {
        input: input ?? {
          index: './index.ts?foo=bar',
        },
        output: {
          path: path.join('dist', p),
          filename: 'index.mjs',
        },
        lazyCompilation: false,
        sourcemap: false,
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
          expect(param.source).toBe('./index.ts?foo=bar');
          expect(param.importer).toBe(null);
          expect(param.kind).toBe('entry');

          return {
            resolvedPath,
            query: [['foo', 'bar']],
            sideEffects: false,
            external: false,
            meta: {},
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
    expect(result.default).toBe(2);
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
            moduleType: 'ts',
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
    expect(result.default).toBe(33);
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
          expect(param.moduleType).toBe('ts');
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
    expect(result.default).toBe(44);
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe(44);
  }
});

test('Js Plugin Execution - full', async () => {
  const root = getJsPluginsFixturesDir();
  const resolvedPath = path.join(root, 'resolved.ts');
  const compiler = await getCompiler('resolve', [
    {
      name: 'test-full',
      priority: 1000,
      resolve: {
        filters: {
          sources: ['.*'],
          importers: ['.ts$'],
        },
        executor: async (param) => {
          console.log(param);

          if (param.source === './resolved?lang=ts&index=1') {
            return {
              resolvedPath,
              query: [
                ['lang', 'ts'],
                ['index', '1'],
              ],
              sideEffects: false,
              external: false,
              meta: {},
            };
          } else {
            return {
              resolvedPath,
              query: [],
              sideEffects: false,
              external: false,
              meta: {},
            };
          }
        },
      },
      load: {
        filters: {
          resolvedPaths: [path.join(root, 'index.ts').replaceAll('\\', '\\\\')],
        },
        executor: async (param) => {
          return {
            content: 'import "./resolved?lang=ts&index=1"; export default 2;',
            moduleType: 'ts',
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
    expect(result.default).toBe(2);
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe(2);
  }
});
