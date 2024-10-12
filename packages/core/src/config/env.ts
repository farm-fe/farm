/**
 * The following is modified based on source found in
 * https://github.com/vitejs/vite/blob/main/packages/vite/src/node/env.ts
 *
 * MIT License
 * Copyright (c) 2019-present, Yuxi (Evan)
 * https://github.com/vitejs/vite/blob/main/LICENSE
 *
 */

import fs from 'node:fs';
import path from 'node:path';

import { parse } from 'dotenv';
import { type DotenvPopulateInput, expand } from 'dotenv-expand';
import { arraify, normalizePath, tryStatSync } from '../utils/index';

export function loadEnv(
  mode: string,
  envDir: string,
  prefixes: string | string[] = ['FARM_', 'VITE_']
): Record<string, string> {
  if (mode === 'local') {
    throw new Error(
      `"local" cannot be used as a mode name because it conflicts with ` +
        `the .local postfix for .env files.`
    );
  }
  prefixes = arraify(prefixes);
  const env: Record<string, string> = {};
  const envFiles = getEnvFilesForMode(mode, envDir);
  const parsed = Object.fromEntries(
    envFiles.flatMap((filePath) => {
      if (!tryStatSync(filePath)?.isFile()) return [];
      return Object.entries(parse(fs.readFileSync(filePath)));
    })
  );
  const processEnv = { ...process.env } as DotenvPopulateInput;
  expand({ parsed, processEnv });

  // only keys that start with prefix are exposed to client
  for (const [key, value] of Object.entries(parsed)) {
    if (prefixes.some((prefix) => key.startsWith(prefix))) {
      env[key] = value;
    }
  }
  for (const key in process.env) {
    if (
      prefixes.some((prefix) => key.startsWith(prefix)) &&
      key !== 'FARM_LIB_CORE_PATH'
    ) {
      env[key] = process.env[key] as string;
    }
  }
  return env;
}

export function getExistsEnvFiles(mode: string, envDir: string): string[] {
  const envFiles = getEnvFilesForMode(mode, envDir);
  return envFiles.filter((filePath) => tryStatSync(filePath)?.isFile());
}

export type CompilationMode = 'development' | 'production';

export function setProcessEnv(mode: CompilationMode) {
  process.env.NODE_ENV = mode;
}

export const isDisableCache = () => !!process.env.DISABLE_CACHE;

export function getEnvFilesForMode(mode: string, envDir: string): string[] {
  return [`.env`, `.env.local`, `.env.${mode}`, `.env.${mode}.local`].map(
    (file) => normalizePath(path.join(envDir, file))
  );
}
