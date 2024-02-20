import { pathToFileURL } from 'url';
import { expect, test } from 'vitest';
import { getCompiler, getOutputFilePath } from './common.js';
import { ResourcePotInfo } from '../../src/index.js';

test('Js Plugin Execution - renderResourcePot', async () => {
  const hookName = 'render-resource-pot';
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: 'test-render-resource-pot',
        priority: 1000,
        renderResourcePot: {
          filters: {
            moduleIds: ['^index.ts\\?foo=bar$'],
            resourcePotTypes: ['js']
          },
          executor: async (param) => {
            expect(param.content).toContain('render-resource-pot-return-value');
            expect(param.sourceMapChain).toEqual([]);
            console.log(param.resourcePotInfo);
            expect(param.resourcePotInfo).toEqual({
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
                isImplicitEntry: false,
                custom: {}
              }
            } as ResourcePotInfo);
            calledHooks.push('renderResourcePot');
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

  await compiler.compile();
  await compiler.writeResourcesToDisk();

  expect(calledHooks).toEqual(['renderResourcePot']);

  const outputFilePath = getOutputFilePath('', hookName);

  if (process.platform === 'win32') {
    const result = await import(pathToFileURL(outputFilePath).toString());
    expect(result.default).toBe('1');
  } else {
    const result = await import(outputFilePath);
    expect(result.default).toBe('1');
  }
});
