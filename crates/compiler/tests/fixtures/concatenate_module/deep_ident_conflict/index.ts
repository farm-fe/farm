import r from 'node:fs';
import { readFileSync } from './dep';

function main(e) {
  console.log(readFileSync('./dep.js'), r.readFileSync('./dep.js'), e);
}

main('hello world');