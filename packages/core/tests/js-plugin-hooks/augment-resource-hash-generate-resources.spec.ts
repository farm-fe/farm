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
        name: 'test-augmentResourceHash',
        priority: 1000,
        augmentResourceHash: {
          filters: {
            moduleIds: ['^index.ts\\?foo=bar$'],
            resourcePotTypes: ['js']
          },
          executor: async (resourcePotInfo) => {
            console.log(resourcePotInfo);
            // originalLength maybe 51 or 52, it's 52 on windows and 51 on linux because of line ending \r\n or \n
            // we normalize it to 51 to make the test pass on both platforms
            if (
              resourcePotInfo.modules['index.ts?foo=bar'].originalLength == 52
            ) {
              resourcePotInfo.modules['index.ts?foo=bar'].originalLength = 51;
            }

            expect(resourcePotInfo).matchSnapshot();

            calledHooks.push('augmentResourceHash');
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

  expect(calledHooks).toEqual(['augmentResourceHash', 'finalizeResources']);
});
