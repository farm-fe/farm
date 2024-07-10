import module from 'module';
import type { UserConfig } from '@farmfe/core';
import lodashMerge from 'lodash/merge';

function defineConfig(config: UserConfig) {
  lodashMerge(config, {
    compilation: {
      input: {
        main: './main.tsx',
      },
      external: module.builtinModules,
    },
  });
  return config;
}

import('lodash/debounce').then((debounce) => {
  console.log(debounce);
});

export default defineConfig({});
