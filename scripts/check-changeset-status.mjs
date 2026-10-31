import fs from 'node:fs';
import path from "node:path";
import { fileURLToPath } from 'node:url';

// Failure to read state must never become permission to publish.
export function changesetStatus(root = process.cwd()) {
  const directory = path.join(root, '.changeset');
  const files = fs.readdirSync(directory).filter(file => file.endsWith('.md') && file !== 'README.md');
  const preFile = path.join(directory, 'pre.json');
  if (!fs.existsSync(preFile)) return { state: files.length ? 'pending' : 'empty', pending: files.length };
  const pre = JSON.parse(fs.readFileSync(preFile, 'utf8'));
  if (!['pre', 'exit'].includes(pre.mode) || !Array.isArray(pre.changesets) || !pre.changesets.every(id => typeof id === 'string')) {
    throw new Error('Invalid Changesets prerelease state');
  }
  if (pre.mode === 'exit') return { state: 'pending', pending: files.length, reason: 'stable-version-preparation' };
  const consumed = new Set(pre.changesets);
  const pending = files.filter(file => !consumed.has(file.slice(0, -3))).length;
  return { state: 'pending', pending, reason: 'active-prerelease' };
}

if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try {
    const result = changesetStatus();
    console.log(JSON.stringify(result));
    if (process.env.GITHUB_OUTPUT) fs.appendFileSync(process.env.GITHUB_OUTPUT, `state=${result.state}\n`);
  } catch (error) {
    console.error(JSON.stringify({ state: 'error', message: error.message }));
    process.exitCode = 2;
  }
}

