import type { UserConfig } from './src/index.js';

export default <UserConfig>{
  compilation: {
    input: {
      index: 'src/index.ts'
    },
    output: {
      path: 'cjs',
      format: 'cjs',
      targetEnv: 'node',
      entryFilename: 'index.cjs'
    },
    external: [
      '.node$',
      '@farmfe/core',
      'bufferutil',
      'utf-8-validate',
      'fsevents',
      'browserslist-generator'
    ],
    presetEnv: false,
    minify: false,
    sourcemap: false,
    persistentCache: false,
    partialBundling: {
      enforceResources: [
        {
          name: 'index',
          test: ['.+']
        }
      ]
    }
  }
};
