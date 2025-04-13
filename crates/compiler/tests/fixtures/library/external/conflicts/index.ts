import { existsSync, createRequire } from './bar';
import './foo';

import * as ns from './bar';

console.log('index readFileSync', existsSync('index'));

console.log(ns);
// TODO fix this test
console.log(createRequire);