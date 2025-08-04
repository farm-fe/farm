import type { UserConfig } from './src/index.js';

export default (<UserConfig>{
  compilation: {
    input: {
      index: 'src/index.ts'
    },
    output: {
      path: 'dist/cjs',
      format: 'cjs',
      targetEnv: 'library',
      entryFilename: 'index.cjs'
    },
    external: [
      'farm',
      'chokidar',
      'farm-browserslist-generator',
      'farm-.*',
      './farm.*.node$',
      './farm.*.cjs',
      '@farmfe/plugin-.*'
    ].map((id) => `^${id}$`),
    comments: true,
    presetEnv: false,
    minify: false,
    sourcemap: false,
    persistentCache: false,
    progress: false
  }
});
