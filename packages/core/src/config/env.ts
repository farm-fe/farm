import fs from 'node:fs';
import path from 'node:path';
import { parse } from 'dotenv';
import { expand } from 'dotenv-expand';
import { tryStatSync } from '../utils/index.js';

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
      if (!tryStatSync(filePath)?.isFile()) return [];

      return Object.entries(parse(fs.readFileSync(filePath)));
    })
  );
  for (const [key, value] of Object.entries(parsed)) {
    if (key.startsWith(prefix)) {
      env[key] = value;
    }
  }

  for (const key in process.env) {
    if (key.startsWith(prefix)) {
      env[key] = process.env[key] as string;
    }
  }
  // `expand` patched in patches/dotenv-expand@9.0.0.patch
  expand({ parsed });
  return env;
}
