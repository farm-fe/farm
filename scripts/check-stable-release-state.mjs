import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export function assertStableReleaseState(root) {
  const changesets = path.join(root, '.changeset');
  if (fs.existsSync(path.join(changesets, 'pre.json'))) throw new Error('Stable publication blocked: prerelease state still exists. Review the beta exit and version PR first.');
  const pending = fs.readdirSync(changesets).filter(name => name.endsWith('.md') && name !== 'README.md');
  if (pending.length) throw new Error(`Stable publication blocked: ${pending.length} unconsumed changesets require the reviewed version PR before the final candidate is sealed.`);
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) assertStableReleaseState(fileURLToPath(new URL('../', import.meta.url)));
