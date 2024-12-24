import cjs, { cjsName } from './cjs';
import esm, { esmName } from './esm';
import bundle2, { bundle2Name } from './bundle2';
import { readFile } from 'node:fs';

console.log({ cjs: { cjs, cjsName }, readFile, esm: { esm, esmName }, bundle2: { bundle2, bundle2Name }  }, 'bar.ts');