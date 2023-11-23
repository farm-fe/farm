import module from 'module';

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
      ...module.builtinModules.map((m) => `^${m}$`),
      ...module.builtinModules.map((m) => `^node:${m}$`),
      '.node$',
      '@farmfe/core',
      'bufferutil',
      'utf-8-validate'
    ],
    presetEnv: false,
    minify: false,
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
