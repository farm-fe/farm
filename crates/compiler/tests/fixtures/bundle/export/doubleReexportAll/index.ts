
import { readFile } from './reexport';
import { Worker } from './foo'

console.log({ readFile, Worker });

export * from './reexport';