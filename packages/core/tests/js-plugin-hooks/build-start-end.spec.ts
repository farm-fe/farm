import path from 'path';
import { pathToFileURL } from 'url';
import { expect, test } from 'vitest';
import {
  getCompiler,
  getJsPluginsFixturesDir,
  getOutputFilePath
} from './common.js';

test('Js Plugin Execution - buildStart/buildEnd', async () => {
  const hookName = 'build-start-end';
  const root = getJsPluginsFixturesDir(hookName);
  const resolvedPath = path.join(root, 'index.ts');
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: 'test-build-start-end',
        priority: 1000,
        buildStart: {
          executor: async (_, ctx) => {
            console.log('buildStart');
            const result = await ctx.resolve(
              {
                source: './index.ts',
                importer: null,
                kind: { entry: 'index' }
              },
              {
                caller: 'test-resolve',
                meta: {}
              }
            );
            expect(result.resolvedPath).toBe(resolvedPath);
            calledHooks.push('buildStart');
          }
        },
        buildEnd: {
          executor: async (_, ctx) => {
            const result = await ctx.resolve(
              {
                source: './index.ts',
                importer: null,
                kind: 'import'
              },
              {
                caller: 'test-resolve',
                meta: {}
              }
            );
            expect(result.resolvedPath).toBe(resolvedPath);
            calledHooks.push('buildEnd');
          }
        }
      }
    ],
    hookName
  );

  await compiler.compile();
  await compiler.writeResourcesToDisk();

  expect(calledHooks).toEqual(['buildStart', 'buildEnd']);

  const outputFilePath = getOutputFilePath('', hookName);

  const filePath =
    process.platform === 'win32'
      ? decodeURIComponent(pathToFileURL(outputFilePath).toString())
      : outputFilePath;

  const result = await import(filePath);
  expect(result.default).toBe(1);
});
