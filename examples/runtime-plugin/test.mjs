import assert from 'node:assert';
import res from './dist/index.mjs';

assert.strictEqual((await res).default, 'dynamic-replaced.ts');