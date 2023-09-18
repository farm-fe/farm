import fs from 'node:fs';
import path from 'node:path';

import { parse } from 'dotenv';
import { expand } from 'dotenv-expand';
import { arraify, getFileSystemStats } from '../utils/index.js';

export function loadEnv(
  mode: string,
  envDir: string,
  prefixes: string | string[] = 'FARM_'
): Record<string, string> {
  const env: Record<string, string> = {};
  const envFiles = [`.env`, `.env.local`, `.env.${mode}`, `.env.${mode}.local`];
  const parsed = Object.fromEntries(
    envFiles.flatMap((file) => {
      const filePath = path.join(envDir, file);
      if (!getFileSystemStats(filePath)?.isFile()) return [];
      return Object.entries(parse(fs.readFileSync(filePath)));
    })
  );
  console.log(parsed);

  expand({ parsed });
  console.log(expand({ parsed }));

  // For security reasons, we won't get inline env variables.
  // Do not inject project process.env by default, cause it's unsafe
  prefixes = arraify(prefixes);
  for (const [key, value] of Object.entries(parsed)) {
    if (prefixes.some((prefix) => key.startsWith(prefix))) {
      env[key] = value;
    }
  }
  return env;
}

export type CompilationMode = 'development' | 'production';

export function setProcessEnv(mode: CompilationMode) {
  if (!process.env.NODE_ENV) {
    process.env.NODE_ENV = mode;
  }
}
