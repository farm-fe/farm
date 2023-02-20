import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import walkdir from 'walkdir';

export const TEMPLATES_DIR = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  '..',
  'templates'
);

export function copyFiles(
  source: string,
  dest: string,
  callback?: (content: string) => string
): void {
  walkdir(source, { sync: true }, (p, stat) => {
    if (stat.isFile()) {
      const content = readFileSync(p).toString();
      const newContent = callback?.(content) ?? content;

      const relativePath = path.relative(source, p);
      const destPath = path.join(dest, relativePath);

      if (!existsSync(path.dirname(destPath))) {
        mkdirSync(path.dirname(destPath), { recursive: true });
      }

      writeFileSync(destPath, newContent);
    }
  });
}
