// test that the script is working
import config, { lodashMerge } from './dist/index.mjs';

console.log(config, lodashMerge({}, { a: 1 }));