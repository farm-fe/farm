import { JsPlugin } from '@farmfe/core';
import fs from 'fs';
import sass from 'sass';

export default function farmSassPlugin(): JsPlugin {
  let farmConfig = null;
  return {
    name: 'farm-sass-plugin',
    load: {
      filters: {
        resolvedPaths: ['.scss$'],
      },
      async executor(param, context, hookContext) {
        const { resolvedPath } = param;
        let source = '';

        try {
          source = await fs.promises.readFile(resolvedPath, 'utf-8');
        } catch (error) {
          console.log(error);
        }

        return {
          content: source,
          moduleType: 'css',
        };
      },
    },

    transform: {
      filters: {
        resolvedPaths: ['.scss$'],
      },
      async executor(param, context, hookContext) {
        try {
          const { css, sourceMap } = sass.compileString(param.content, {
            sourceMap: true,
          });
          return {
            content: css,
            moduleType: 'css',
            sourceMap: JSON.stringify(sourceMap),
          };
        } catch (error) {
          console.error(error);
        }
        return {
          content: '',
          moduleType: 'css',
        };
      },
    },
  };
}
