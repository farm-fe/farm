import module from 'module';

import { UserConfig } from '@farmfe/core';

export default {
  compilation: {
    input: {
      main: './main.tsx',
    },
    external: module.builtinModules,
  },
} as UserConfig;
