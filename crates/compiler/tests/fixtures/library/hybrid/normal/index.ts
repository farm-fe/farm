import fs from 'node:fs';
const os = require('node:os');

console.log(fs.read, os.cpus);

// default export
export default {
  read: fs.read,
  c: 1,
};

export const foo = 'foo';
export const bar = 'bar';