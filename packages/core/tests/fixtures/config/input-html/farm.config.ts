import { builtinModules } from 'module';

export default {
  compilation: {
    input: {
      index: 'index.html'
    },
    output: {
      targetEnv: 'browser-legacy'
    },
    external: builtinModules
  }
};
