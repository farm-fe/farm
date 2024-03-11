import { expect, test } from 'vitest';
import { getCompiler } from './common.js';
import { ResourcePotInfo } from '../../src/index.js';

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
            expect(keys).toEqual(['FARM_RUNTIME_runtime', 'index.cc2ddd27.js']);
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
            expect(resourcePotInfo).toEqual({
              id: 'index_eab4_js',
              name: 'index_eab4',
              resourcePotType: 'js',
              moduleIds: ['index.ts?foo=bar'],
              map: null,
              modules: {
                'index.ts?foo=bar': {
                  id: 'index.ts?foo=bar',
                  renderedContent:
                    '"use strict";\n' +
                    'Object.defineProperty(exports, "__esModule", {\n' +
                    '    value: true\n' +
                    '});\n' +
                    'Object.defineProperty(exports, "default", {\n' +
                    '    enumerable: true,\n' +
                    '    get: function() {\n' +
                    '        return _default;\n' +
                    '    }\n' +
                    '});\n' +
                    "const _default = 'render-resource-pot-return-value';\n",
                  renderedMap: null,
                  renderedLength: 257,
                  originalLength: 51
                }
              },
              data: {
                dynamicImports: [],
                exports: [],
                imports: [],
                importedBindings: {},
                isDynamicEntry: false,
                isEntry: true,
                isImplicitEntry: false
              },
              custom: {}
            } as ResourcePotInfo);
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
            expect(keys).toEqual(['FARM_RUNTIME_runtime', 'index.3dc3d75a.js']);
            expect(param.resourcesMap['index.3dc3d75a.js'].name).toBe(
              'index.3dc3d75a.js'
            );
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
