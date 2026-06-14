import { join } from 'node:path';
import { expect, test } from 'vitest';
import { getCompiler } from './common.js';

test('Js Plugin Execution - updateModules', async () => {
  const hookName = 'update-modules';
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: `test-${hookName}`,
        priority: 1000,
        updateModules: {
          executor: async (param) => {
            console.log(param);
            calledHooks.push(hookName);

            expect(param.paths).toEqual([
              [indexPath, 'updated'],
              ['test2', 'added'],
              ['test3', 'removed']
            ]);
          }
        }
      }
    ],
    hookName,
    {
      index: 'index.ts'
    },
    undefined,
    {
      sourcemap: true
    }
  );
  const root = compiler.config.compilation?.root;
  if (!root) {
    throw new Error('Expected compiler root to be defined');
  }
  const indexPath = join(root, 'index.ts');

  console.log('compile');
  await compiler.compile();
  console.log('compile end');

  await compiler.update([
    {
      path: indexPath,
      type: 'updated'
    },
    {
      path: 'test2',
      type: 'added'
    },
    {
      path: 'test3',
      type: 'removed'
    }
  ]);

  expect(calledHooks).toEqual([hookName]);
});

test('Js Plugin Execution - updateModules result replaces update paths', async () => {
  const hookName = 'update-modules';
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: `test-${hookName}-replace`,
        priority: 1000,
        updateModules: {
          executor: async () => {
            calledHooks.push('replace');
            return [[indexPath, 'updated']];
          }
        }
      },
      {
        name: `test-${hookName}-observe`,
        priority: 999,
        updateModules: {
          executor: async (param) => {
            calledHooks.push('observe');
            expect(param.paths).toEqual([[indexPath, 'updated']]);
          }
        }
      }
    ],
    hookName,
    {
      index: 'index.ts'
    },
    undefined,
    {
      sourcemap: true
    }
  );
  const root = compiler.config.compilation?.root;
  if (!root) {
    throw new Error('Expected compiler root to be defined');
  }
  const indexPath = join(root, 'index.ts');

  await compiler.compile();

  await compiler.update([
    {
      path: indexPath,
      type: 'updated'
    },
    {
      path: 'test2',
      type: 'added'
    }
  ]);

  expect(calledHooks).toEqual(['replace', 'observe']);
});
