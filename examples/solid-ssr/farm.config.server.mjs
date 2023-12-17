import { builtinModules } from 'node:module';
import solid from '@farmfe/js-plugin-solid';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './src/server.tsx'
    },
    output: {
      path: './dist',
      targetEnv: 'node',
      format: 'esm'
    },
    resolve: {},
    external: [...builtinModules.map((m) => `^${m}$`)],
    css: {
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
    treeShaking: false,
    persistentCache: false,
    minify: false
  },
  plugins: [solid({ ssr: true }), '@farmfe/plugin-sass']
};
