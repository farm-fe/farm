import type { UserConfig } from './src/index.js';

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
      'chokidar',
      'farm-browserslist-generator',
      '@farmfe/core-.*',
      './farm.*.node$'
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
  }
});
