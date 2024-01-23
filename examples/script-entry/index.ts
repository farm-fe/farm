import module from 'module';
import { UserConfig } from '@farmfe/core';
import lodashMerge from 'lodash/merge';

function defineConfig(config: UserConfig) {
  lodashMerge(config, {
    compilation: {
      input: {
        main: './main.tsx'
      },
      external: module.builtinModules
    }
  });
  return config;
}

import('lodash/debounce').then((debounce) => {
  const ld = debounce;
  console.log('ld is', ld);
});

export default defineConfig({});

export { lodashMerge };
