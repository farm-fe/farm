import { existsSync, mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import Module from 'node:module';
import { fileURLToPath, pathToFileURL } from 'node:url';
import walkdir from 'walkdir';
import type { start, build } from '@farmfe/core';

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
      const content = readFileSync(p).toString('utf-8');
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

export function resolveCore(cwd: string): Promise<{
  start: typeof start;
  build: typeof build;
}> {
  const require = Module.createRequire(path.join(cwd, 'package.json'));
  const farmCorePath = require.resolve('@farmfe/core');

  if (process.platform === 'win32') {
    return import(pathToFileURL(farmCorePath).toString());
  }

  return import(farmCorePath);
}
