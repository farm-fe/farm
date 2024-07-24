import type { UserConfig } from '@farmfe/core';
import dts from '@farmfe/js-plugin-dts';
import path from 'node:path';

const config: UserConfig = {
  compilation: {
    input: {
      index: 'index.ts'
    },
    output: {
      path: 'dist/cjs',
      entryFilename: '[entryName].cjs',
      targetEnv: 'library-node',
      format: 'cjs'
    },
    lazyCompilation: false,
    resolve: {
      alias: {
        '@/': path.join(process.cwd(), 'src')
      }
    },
    partialBundling: {
      enforceResources: [
        {
          name: 'index',
          test: ['.+']
        }
      ]
    },
    minify: false,
    sourcemap: true
  },
  plugins: [
    dts({
      tsConfigPath: './tsconfig.json'
    })
  ]
};

export default config;
