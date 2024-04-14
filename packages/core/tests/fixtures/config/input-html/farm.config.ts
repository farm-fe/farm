import { builtinModules } from 'module';

export default {
  compilation: {
    input: {
      index: 'index.html'
    },
    external: builtinModules
  }
};
