import { fs, fs1, os } from './dep';

console.log('dep', fs, fs1, os);

export * as os from 'node:os'