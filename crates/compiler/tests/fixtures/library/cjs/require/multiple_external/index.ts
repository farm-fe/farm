import { fs, path } from './dep.cjs';
import { fs as fs1, path as path1 } from './dep1.cjs';

console.log('dep.cjs', fs, path);
console.log('dep1.cjs', fs1, path1);