import json1 from './json1.json';

// eslint-disable-next-line @typescript-eslint/no-var-requires
const json2 = require('./json2.json');

const json1Name = json1.name;
const json2Name = json2.name;

export default { json1Name, json2Name };
