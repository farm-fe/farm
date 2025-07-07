import path from 'path';
import { expect, test } from 'vitest';
import { JsPlugin } from '../src/index.js';
import {
  getFixturesDir,
  getCompiler as getInternalCompiler,
  getOutputResult
} from './common.js';

function getJsPluginsFixturesDir() {
  return path.resolve(getFixturesDir(), 'js-plugins');
}

function getOutputFilePath(p: string) {
  const root = getJsPluginsFixturesDir();
  return path.join(root, 'dist', p, 'index.mjs');
}

function getCompiler(p: string, plugins: JsPlugin[]) {
  const root = getJsPluginsFixturesDir();
  return getInternalCompiler(root, p, plugins);
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
          importers: ['None']
        },
        executor: async (param) => {
          console.log(param);
          expect(param.source).toBe('./index.ts?foo=bar');
          expect(param.importer).toBe(null);
          expect(param.kind).toEqual({ entry: 'index' });
          console.log(resolvedPath);

          return {
            resolvedPath,
            query: [['foo', 'bar']]
          };
        }
      }
    }
  ]);

  await compiler.compile();
  await compiler.writeResourcesToDisk();

  const outputFilePath = getOutputFilePath('resolve');
  const result = await getOutputResult(outputFilePath);
  expect(result.default).toBe(2);
});

test('Js Plugin Execution - load', async () => {
  const root = getJsPluginsFixturesDir();
  const compiler = await getCompiler('load', [
    {
      name: 'test-load',
      priority: 1000,
      load: {
        filters: {
          resolvedPaths: [path.join(root, 'index.ts').replaceAll('\\', '\\\\')]
        },
        executor: async (param) => {
          console.log(param);
          return {
            content: 'export default 33;',
            moduleType: 'ts'
          };
        }
      }
    }
  ]);

  await compiler.compile();
  await compiler.writeResourcesToDisk();

  const outputFilePath = getOutputFilePath('load');
  const result = await getOutputResult(outputFilePath);
  expect(result.default).toBe(33);
});

test('Js Plugin Execution - transform', async () => {
  const root = getJsPluginsFixturesDir();
  const compiler = await getCompiler('transform', [
    {
      name: 'test-transform',
      priority: 1000,
      transform: {
        filters: {
          resolvedPaths: [path.join(root, 'index.ts').replaceAll('\\', '\\\\')]
        },
        executor: async (param) => {
          console.log(param);
          expect(param.moduleType).toBe('ts');
          return {
            content: 'export default 44;'
          };
        }
      }
    }
  ]);

  await compiler.compile();
  await compiler.writeResourcesToDisk();

  const outputFilePath = getOutputFilePath('transform');
  const result = await getOutputResult(outputFilePath);
  expect(result.default).toBe(44);
});

test('Js Plugin Execution - full', async () => {
  const root = getJsPluginsFixturesDir();
  const resolvedPath = path.join(root, 'resolved.ts');

  const calledHooks: string[] = [];

  const compiler = await getCompiler('full', [
    {
      name: 'test-full',
      priority: 1000,
      buildStart: {
        executor: async () => {
          calledHooks.push('buildStart');
        }
      },
      resolve: {
        filters: {
          sources: ['.*'],
          importers: ['.ts$']
        },
        executor: async (param) => {
          console.log(param);

          if (param.source === './resolved?lang=ts&index=1') {
            return {
              resolvedPath,
              query: [
                ['lang', 'ts'],
                ['index', '1']
              ],
              sideEffects: false,
              external: false,
              meta: {}
            };
          } else {
            return {
              resolvedPath,
              query: [],
              sideEffects: false,
              external: false,
              meta: {}
            };
          }
        }
      },
      load: {
        filters: {
          resolvedPaths: [path.join(root, 'index.ts').replaceAll('\\', '\\\\')]
        },
        executor: async () => {
          return {
            content: 'import "./resolved?lang=ts&index=1"; export default 2;',
            moduleType: 'ts'
          };
        }
      },
      buildEnd: {
        executor: async () => {
          calledHooks.push('buildEnd');
        }
      },
      finalizeResources: {
        executor: async (params) => {
          expect(params.config).toBeTruthy();
          expect({
            resourcesMap: Object.fromEntries(
              Object.entries(params.resourcesMap).map(([k, v]) => [
                k,
                {
                  ...v,
                  bytes: [] as any[],
                  code: Buffer.from(v.bytes).toString('utf-8')
                }
              ])
            )
          }).matchSnapshot();
          calledHooks.push('finalizeResources');
          return params.resourcesMap;
        }
      },
      finish: {
        executor: async () => {
          calledHooks.push('finish');
        }
      }
    }
  ]);

  await compiler.compile();
  await compiler.writeResourcesToDisk();

  expect(calledHooks).toEqual([
    'buildStart',
    'buildEnd',
    'finalizeResources',
    'finish'
  ]);

  const outputFilePath = getOutputFilePath('full');
  const result = await getOutputResult(outputFilePath);
  expect(result.default).toBe(2);
});
