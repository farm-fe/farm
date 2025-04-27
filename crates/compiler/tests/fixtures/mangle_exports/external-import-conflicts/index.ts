import { existsSync, createRequire } from './bar';
import './foo';

import * as ns from './bar';

import { unresolved } from './zoo';
import { unresolvedDeep } from './zoo';

import { unresolvedDeep as unresolvedDeepConflict } from '/external/deep/unresolved';

console.log('index readFileSync', existsSync('index'));

console.log(unresolved, unresolvedDeep, unresolvedDeepConflict);

console.log(ns);
// TODO fix this test
console.log(createRequire);