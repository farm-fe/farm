import { readFile, resolve, dep3 } from './dep1'
import { cpus, spawn } from './dep2'

console.log(readFile, resolve, cpus, spawn, dep3);