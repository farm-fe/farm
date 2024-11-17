import { resolve } from 'node:path';
import { defineConfig } from '@farmfe/core';
import farmJsPluginLess from '@farmfe/js-plugin-less';
import farmJsPluginSvgr from '@farmfe/js-plugin-svgr';

export default defineConfig((env) => {
  return {
    compilation: {
      input: {
        index: './index.html'
      },
      sourcemap: false,
      presetEnv: false,
      concatenateModules: true,
      // persistentCache: false,
      resolve: {
        symlinks: true,
        alias: {
          '@': resolve(process.cwd(), './src'),
          'react-dom': resolve(process.cwd(), './node_modules/react-dom'),
          react: resolve(process.cwd(), './node_modules/react')
        }
      },
      // minify: false,
      // mode: 'development',
      // persistentCache: false,
      output: {
        path: './build',
        filename: 'assets/[resourceName].[contentHash].[ext]',
        assetsFilename: 'static/[resourceName].[contentHash].[ext]'
      },
      partialBundling: {
        targetMinSize: 1024 * 2000,
        groups: [
          {
            name: 'components',
            test: ['src/components/.+'],
            enforce: true,
          },
          {
            name: 'xxxx',
            test: ['src/pages/.+']
          }
        ]
      },
      progress: true
    },
    server: {
      cors: true,
      port: 6290
    },
    plugins: [
      [
        '@farmfe/plugin-react',
        {
          refresh: env.mode === 'development',
          development: env.mode === 'development'
        }
      ],
      farmJsPluginLess(),
      farmJsPluginSvgr()
    ]
  };
});
