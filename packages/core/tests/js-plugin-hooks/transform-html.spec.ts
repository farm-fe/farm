import { expect, test } from 'vitest';
import { getCompiler } from './common.js';

test('Js Plugin Execution - transformHtml', async () => {
  const hookName = 'transform-html';
  const calledHooks: string[] = [];
  const compiler = await getCompiler(
    '',
    [
      {
        name: 'test-transform-html-pre',
        transformHtml: {
          order: 0, // 0 means call this hook before parse and generate resources
          executor: async ({ htmlResource }) => {
            const code = Buffer.from(htmlResource.bytes as any).toString();
            calledHooks.push('transformHtmlPre');
            const replacedCode = code
              .replace('{head}', '<meta head />')
              .replace('{style}', '<meta style />')
              .replace('{ssr}', '<meta ssr />');
            return {
              ...htmlResource,
              bytes: [...Buffer.from(replacedCode)]
            };
          }
        }
      },
      {
        name: 'test-transform-html-post',
        transformHtml: {
          order: 2, // 2 means call this hook after parse and generate resources
          executor: async ({ htmlResource }) => {
            calledHooks.push('transformHtmlPost');
            const replacedCode = Buffer.from(htmlResource.bytes as any)
              .toString()
              .replace(/<meta (\w+?)>/g, '{$1}');
            htmlResource.bytes = [...Buffer.from(replacedCode)];
            return htmlResource;
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
    },
    {
      runtime: {
        isolate: false
      }
    }
  );

  await compiler.compile();

  expect(calledHooks).toEqual(['transformHtmlPre', 'transformHtmlPost']);

  const resourcesMap = compiler.resources();
  const html = resourcesMap['index.html'];
  expect(Buffer.from(html).toString()).matchSnapshot();
});
