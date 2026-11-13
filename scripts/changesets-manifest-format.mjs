import fs from 'node:fs';
import path from 'node:path';
import { execFileSync } from 'node:child_process';
import { isDeepStrictEqual } from 'node:util';

// Keep formatting-only Changesets rewrites out of the version PR. Never erase
// semantic changes: those must still reach the existing source-change gate.
export function snapshotUnpublishedManifests(root, baseline) {
  const published = new Set([...baseline.npm.primary, ...baseline.npm.payloads].map(row => row.manifestPath));
  const tracked = execFileSync('git', ['ls-files', '-z'], { cwd: root, encoding: 'utf8' }).split('\0');
  return tracked.filter(file => (file === 'package.json' || file.endsWith('/package.json')) && !published.has(file))
    .flatMap(file => {
      const absolute = path.join(root, file), stat = fs.lstatSync(absolute);
      if (!stat.isFile()) return [];
      const bytes = fs.readFileSync(absolute, 'utf8');
      return [{ file, bytes, mode: stat.mode, value: JSON.parse(bytes) }];
    });
}

export function restoreUnchangedManifestFormatting(root, snapshots) {
  const restored = [];
  for (const { file, bytes, mode, value } of snapshots) {
    const absolute = path.join(root, file);
    if (!fs.existsSync(absolute)) continue;
    const stat = fs.lstatSync(absolute);
    if (!stat.isFile() || stat.mode !== mode) continue;
    const current = fs.readFileSync(absolute, 'utf8');
    if (current === bytes) continue;
    // Invalid JSON deliberately fails instead of being silently discarded.
    if (!isDeepStrictEqual(JSON.parse(current), value)) continue;
    fs.writeFileSync(absolute, bytes);
    restored.push(file);
  }
  return restored;
}
