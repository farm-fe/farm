import { expect, test } from 'vitest';
import { getCompiler } from './common.js';

test('Js Plugin Execution - transformHtml', async () => {
  const hookName = 'transform-html';
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: 'test-transform-html',
        transformHtml: {
          executor: async ({ htmlResource }) => {
            calledHooks.push('transformHtml');
            const html = Buffer.from(htmlResource.bytes).toString();
            expect(html).toContain('<div id=app-container></div>');
            return {
              ...htmlResource,
              bytes: [
                ...Buffer.from(html.replace('app-container', 'app-container2'))
              ]
            };
          }
        }
      }
    ],
    hookName,
    {
      index: './index.html'
    },
    {
      entryFilename: 'index.html'
    }
  );

  await compiler.compile();

  expect(calledHooks).toEqual(['transformHtml']);

  const resourcesMap = compiler.resources();
  const html = resourcesMap['index.html'];
  expect(Buffer.from(html).toString()).toContain(
    '<div id=app-container2></div>'
  );
});
