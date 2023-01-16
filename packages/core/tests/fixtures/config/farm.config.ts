import { defineFarmConfig } from '../../../src/node/config';
import input from './util';

export default defineFarmConfig({
  compilation: {
    input,
  },
});
