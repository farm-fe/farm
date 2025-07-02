import { expect, test } from 'vitest';
import { getCompiler } from './common.js';

test('Js Plugin Execution - augmentResourceHash', async () => {
  const hookName = 'augment-resource-hash-generate-resources';
  const calledHooks0: string[] = [];
  const compiler0 = await getCompiler(
    '',
    [
      {
        name: 'test-finalize-resources',
        finalizeResources: {
          executor: async (param) => {
            calledHooks0.push('finalizeResources');
            const keys = Object.keys(param.resourcesMap);
            keys.sort();

            expect(keys).matchSnapshot();
            return param.resourcesMap;
          }
        }
      }
    ],
    hookName,
    undefined,
    {
      entryFilename: '[entryName].[hash].js'
    }
  );

  await compiler0.compile();

  expect(calledHooks0).toEqual(['finalizeResources']);

  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: 'test-augmentResourcePotHash',
        priority: 1000,
        augmentResourcePotHash: {
          filters: {
            moduleIds: ['^index.ts\\?foo=bar$'],
            resourcePotTypes: ['js']
          },
          executor: async (resourcePotInfo) => {
            console.log(resourcePotInfo);

            expect(resourcePotInfo).matchSnapshot();

            calledHooks.push('augmentResourcePotHash');
            return 'augmented-hash';
          }
        },
        finalizeResources: {
          async executor(param) {
            calledHooks.push('finalizeResources');
            const keys = Object.keys(param.resourcesMap);
            keys.sort();
            console.log(keys);
            expect(keys).matchSnapshot();
            return param.resourcesMap;
          }
        }
      }
    ],
    hookName,
    undefined,
    {
      entryFilename: '[entryName].[hash].js'
    }
  );

  await compiler.compile();

  expect(calledHooks).toEqual(['augmentResourcePotHash', 'finalizeResources']);
});
