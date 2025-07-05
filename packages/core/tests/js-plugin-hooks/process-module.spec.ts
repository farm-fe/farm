import { expect, test } from 'vitest';
import { getOutputResult } from '../common.js';
import { getCompiler, getOutputFilePath } from './common.js';

test('Js Plugin Execution - processModule', async () => {
  const hookName = 'process-module';
  const calledHooks: string[] = [];
  const calledModules: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: `test-${hookName}`,
        priority: 1000,
        processModule: {
          filters: {
            resolvedPaths: ['^index.ts$']
            // moduleTypes: ['js']
          },
          executor: async (param) => {
            expect(param.content).toContain(`${hookName}-return-value`);

            expect(param).matchSnapshot();
            calledHooks.push(hookName);
            calledModules.push(param.moduleId);

            return {
              content: param.content.replace(`${hookName}-return-value`, '1')
            };
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
  compiler.writeResourcesToDisk();

  expect(calledHooks).toEqual([hookName]);
  expect(calledModules).toEqual(['index.ts']);

  const outputFilePath = getOutputFilePath('', hookName);
  const result = await getOutputResult(outputFilePath);
  expect(result.default).toBe('1');
});
