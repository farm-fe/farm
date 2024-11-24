import { resolve } from 'node:path';
import { defineConfig } from '@farmfe/core';
import farmJsPluginLess from '@farmfe/js-plugin-less';
import farmJsPluginSvgr from '@farmfe/js-plugin-svgr';

export default defineConfig((env) => {
  return {
    compilation: {
      resolve: {
        alias: {
          '@': resolve(process.cwd(), './src'),
        }
      },
      output: {
        path: './build',
        filename: 'assets/[resourceName].[contentHash].[ext]',
        assetsFilename: 'static/[resourceName].[contentHash].[ext]'
      },
      partialBundling: {
        targetMinSize: 1024 * 2
      },
      progress: false
    },
    plugins: [
      [
        '@farmfe/plugin-react',
        {
          refresh: env.mode === 'development',
          development: env.mode === 'development'
        }
      ],
      '@farmfe/plugin-svgr',
      farmJsPluginLess(),
    ]
  };
});
