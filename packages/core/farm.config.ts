import path from 'path';
import type { UserConfig } from './src/index.js';

const VIRTUAL_SUFFIX = '.virtual.farm';

export default (<UserConfig>{
  compilation: {
    input: {
      index: 'src/index.ts'
    },
    output: {
      path: 'dist/cjs',
      format: 'cjs',
      targetEnv: 'node',
      entryFilename: 'index.cjs'
    },
    external: [
      '@farmfe/core',
      // 'bufferutil',
      // 'utf-8-validate',
      // 'fsevents',
      'chokidar',
      'browserslist-generator'
    ].map((id) => `^${id}$`),
    presetEnv: false,
    minify: false,
    sourcemap: false,
    persistentCache: false,
    progress: false,
    partialBundling: {
      enforceResources: [
        {
          name: 'index',
          test: ['.+']
        }
      ]
    }
  },
  plugins: [
    // external external binding
    {
      name: 'external-binding',
      priority: 102,
      resolve: {
        filters: {
          sources: ['(binding|resolve-binding)\\.cjs$'],
          importers: ['binding/index.js', `${VIRTUAL_SUFFIX}$`]
        },
        async executor(param, context, hookContext) {
          if (hookContext?.caller === 'external-binding') {
            return null;
          }

          if (param.importer.endsWith(VIRTUAL_SUFFIX)) {
            const relativePath = path
              .relative(
                process.cwd(),
                param.importer.replace(VIRTUAL_SUFFIX, '')
              )
              .replace(/\\/g, '/');
            console.log('relativePath', relativePath);
            if (
              relativePath === 'binding/binding.cjs' ||
              relativePath === 'binding/resolve-binding.cjs'
            ) {
              return {
                resolvedPath: `../../${relativePath}`,
                external: true
              };
            }
          }

          const result = await context.resolve(param, {
            caller: 'external-binding',
            meta: {}
          });

          if (result) {
            return {
              ...result,
              resolvedPath: result.resolvedPath + VIRTUAL_SUFFIX
            };
          }

          return null;
        }
      },
      load: {
        filters: {
          resolvedPaths: [
            'packages/core/binding/binding.cjs',
            'packages/core/binding/resolve-binding.cjs'
          ]
        },
        async executor({ resolvedPath }) {
          console.log('resolvedPath', resolvedPath);
          if (resolvedPath.endsWith(VIRTUAL_SUFFIX)) {
            const relativePath = path
              .relative(process.cwd(), resolvedPath.replace(VIRTUAL_SUFFIX, ''))
              .replace(/\\/g, '/');

            if (
              relativePath === 'binding/binding.cjs' ||
              relativePath === 'binding/resolve-binding.cjs'
            ) {
              console.log(`module.exports = require('../../${relativePath}');`);
              return {
                content: `module.exports = require('../../${relativePath}');`,
                moduleType: 'js'
              };
            }
          }

          return null;
        }
      }
    }
  ]
});
