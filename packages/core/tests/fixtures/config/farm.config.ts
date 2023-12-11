import { builtinModules } from 'module';
import input from './util.js';

export default {
  compilation: {
    input,
    external: builtinModules
  }
};
