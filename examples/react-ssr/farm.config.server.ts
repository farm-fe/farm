import type { UserConfig } from '@farmfe/core';
import { builtinModules } from 'node:module';

export default <UserConfig>{
  compilation: {
    input: {
      index: './src/index-server.tsx'
    },
    output: {
      path: './dist',
      targetEnv: 'node',
      format: 'cjs',
    },
    external: [...builtinModules.map((m) => `^${m}$`)],
    partialBundling: {
      moduleBuckets: [
        {
          test: ['.+'],
          name: 'index-server'
        }
      ]
    },
    minify: false,
    css: {
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    }
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
};
