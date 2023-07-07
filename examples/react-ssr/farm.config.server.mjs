import { builtinModules } from 'node:module';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index: './src/index-server.tsx'
    },
    output: {
      path: './dist',
      targetEnv: 'node',
      format: 'cjs'
    },
    external: [...builtinModules.map((m) => `^${m}$`)],
    css: {
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    }
  },
  plugins: [
    [
      '@farmfe/plugin-react',
      {
        refresh: false,
        development: false
      }
    ],
    '@farmfe/plugin-sass'
  ]
};
