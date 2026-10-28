import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import { pathToFileURL } from 'node:url';
import { compile } from '@mdx-js/mdx';
import Papa from 'papaparse';
import { parse } from 'yaml';
import type { JsPlugin } from '@farmfe/core';
const markdownRequire = createRequire(new URL('../../../website/package.json', import.meta.url));

/** Thin Farm load hook around the same maintained parsers used at runtime. */
export function data(): JsPlugin {
  return {
    name: 'commerce-js-data',
    load: {
      filters: { resolvedPaths: ['\\.(csv|tsv|yaml|yml|mdx)$'] },
      async executor(param) {
        const content = await fs.readFile(param.resolvedPath, 'utf8');
        if (param.resolvedPath.endsWith('.mdx')) {
          const { default: gfm } = await import(pathToFileURL(markdownRequire.resolve('remark-gfm')).href);
          return {
            content: String(
              await compile(content, { development: false, jsx: false, remarkPlugins: [gfm] }),
            ),
            moduleType: 'js',
          };
        }
        const isDsv = /\.(csv|tsv)$/.test(param.resolvedPath);
        const result = isDsv
          ? Papa.parse(content, {
              header: true,
              skipEmptyLines: true,
              delimiter: param.resolvedPath.endsWith('.tsv') ? '\t' : ',',
            })
          : null;
        if (result?.errors.length)
          throw new Error(JSON.stringify(result.errors));
        const value = result ? result.data : parse(content);
        return {
          content: `export default ${JSON.stringify(value)};`,
          moduleType: 'js',
        };
      },
    },
  };
}
