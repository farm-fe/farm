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
              [join(compiler.config.compilation.root, 'index.ts'), 'updated'],
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

  console.log('compile');
  await compiler.compile();
  console.log('compile end');

  await compiler.update([
    {
      path: join(compiler.config.compilation.root, 'index.ts'),
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
