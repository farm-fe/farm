import { expect, test } from 'vitest';
import { getOutputResult } from '../common.js';
import { getCompiler, getOutputFilePath } from './common.js';

test('Js Plugin Execution - processRenderedResourcePot', async () => {
  const hookName = 'render-resource-pot';
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: 'test-render-resource-pot',
        priority: 1000,
        processRenderedResourcePot: {
          filters: {
            moduleIds: ['^index.ts\\?foo=bar$'],
            resourcePotTypes: ['js']
          },
          executor: async (param) => {
            expect(param.content).toContain('render-resource-pot-return-value');
            expect(param.sourceMapChain).toEqual([]);
            console.log(param);

            expect(param).matchSnapshot();
            calledHooks.push('processRenderedResourcePot');

            return {
              content: param.content.replace(
                'render-resource-pot-return-value',
                '1'
              )
            };
          }
        }
      }
    ],
    hookName
  );

  console.log('compile');
  await compiler.compile();
  console.log('compile end');
  await compiler.writeResourcesToDisk();

  expect(calledHooks).toEqual(['processRenderedResourcePot']);

  const outputFilePath = getOutputFilePath('', hookName);
  const result = await getOutputResult(outputFilePath);
  expect(result.default).toBe('1');
});
