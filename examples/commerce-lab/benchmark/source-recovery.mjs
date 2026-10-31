import fs from 'node:fs/promises';
import path from 'node:path';

export async function restoreSourceProbes(root, backups, edits) {
  const result = { restored: [], unchanged: [], conflicts: [], errors: [] };
  for (const [file, original] of backups) {
    try {
      const target = path.resolve(root, file);
      const relative = path.relative(path.resolve(root), target);
      if (relative === '..' || relative.startsWith(`..${path.sep}`) || path.isAbsolute(relative)) throw new Error('Recovery path escapes workspace');
      const current = await fs.readFile(target, 'utf8');
      if (current === original) { result.unchanged.push(file); continue; }
      if (edits.some(([candidate, from, to]) => candidate === file && original.split(from).length === 2 && current === original.replace(from, to))) {
        await fs.writeFile(target, original);
        result.restored.push(file);
      } else result.conflicts.push(file);
    } catch (error) { result.errors.push({ file, error: error.message }); }
  }
  return result;
}
