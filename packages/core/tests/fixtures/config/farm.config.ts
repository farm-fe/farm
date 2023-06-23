import { builtinModules } from 'module';
import { defineFarmConfig } from '../../../src/config';
import input from './util';

export default defineFarmConfig({
  compilation: {
    input,
    external: builtinModules
  }
});
