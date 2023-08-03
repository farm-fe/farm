import fs from 'node:fs';
import path from 'node:path';

import { parse, config } from 'dotenv';
import { expand } from 'dotenv-expand';
import { getFileSystemStats } from '../utils/index.js';

export function loadEnv(
  mode: string,
  envDir: string,
  prefix = 'FARM_'
): Record<string, string> {
  const env: Record<string, string> = {};
  const envFiles = [`.env`, `.env.${mode}`];
  const parsed = Object.fromEntries(
    envFiles.flatMap((file) => {
      const filePath = path.join(envDir, file);
      if (!getFileSystemStats(filePath)?.isFile()) return [];

      return Object.entries(parse(fs.readFileSync(filePath)));
    })
  );

  // For security reasons, we won't get inline env variables.
  // Do not inject project process.env by default, cause it's unsafe
  for (const [key, value] of Object.entries(parsed)) {
    if (key.startsWith(prefix)) {
      env[key] = value;
    }
  }

  config();
  expand({ parsed });
  return env;
}

export type CompilationMode = 'development' | 'production';

export function setProcessEnv(mode: CompilationMode) {
  if (!process.env.NODE_ENV) {
    process.env.NODE_ENV = mode;
  }
}
