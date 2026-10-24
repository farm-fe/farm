import assert from 'node:assert/strict';
import fs from 'node:fs/promises';
import { fileURLToPath } from 'node:url';

const group = process.env.COMMERCE_GROUP ?? 'js';
assert.ok(['js', 'rust', 'mixed'].includes(group), 'Invalid group');
const file = fileURLToPath(new URL(`../dist/${group}/import.js`, import.meta.url));
const content = await fs.readFile(file, 'utf8');
assert.ok(content.includes('postMessage'), 'Import validation Worker must contain its message response');
console.log(JSON.stringify({ group, workerPresent: true }));
