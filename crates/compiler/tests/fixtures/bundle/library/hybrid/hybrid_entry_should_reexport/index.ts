import fs from 'node:fs';
const os = require('node:os');

console.log(fs.read, os.cpus);

export const name = 'foo';
module.exports.age = 18;