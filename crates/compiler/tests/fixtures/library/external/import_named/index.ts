import './dep';
import { readFile as r1, readFileSync as r2 } from 'node:fs';


const readFile = 1;
const readFileSync = 2;

console.log({ readFile, readFileSync, r1, r2 });