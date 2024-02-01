import * as t from 'component-classes';

import output from './dist/index.mjs';
import assert from 'assert';

console.log(t, output);

const tKeys = Object.keys(t);
const outputKeys = Object.keys(output);

assert.deepEqual(tKeys, outputKeys);
