import cjs, { cjsAge } from './cjs';
import { readFile } from 'node:fs';
import esm, { esmAge } from './esm';
import bundle2, { bundle2Age } from './bundle2';


console.log({ cjs: { cjs, cjsAge }, esm: { esm, esmAge }, bundle2: { bundle2, bundle2Age }, readFile }, 'foo.ts');