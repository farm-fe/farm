import fs from "node:fs";
import path from "node:path";

import { parse } from "dotenv";
import { expand } from "dotenv-expand";
import { arraify, getFileSystemStats } from "../utils/index.js";

// Because of the limitation of dotenv-expand,
// learn from the operation method of vite to dotenv.
/**
 * The following is modified based on source found in
 * https://github.com/vitejs/vite/pull/14391/files
 */
export function loadEnv(
  mode: string,
  envDir: string,
  prefixes: string | string[] = ["FARM_", "VITE_"]
): [env: Record<string, string>, existsEnvFiles: string[]] {
  const env: Record<string, string> = {};
  const existsEnvFiles: string[] = [];
  const envFiles = [`.env`, `.env.local`, `.env.${mode}`, `.env.${mode}.local`];

  const parsed = Object.fromEntries(
    envFiles.flatMap((file) => {
      const filePath = path.join(envDir, file);
      if (!getFileSystemStats(filePath)?.isFile()) return [];
      existsEnvFiles.push(filePath);
      return Object.entries(parse(fs.readFileSync(filePath)));
    })
  );
  expand({ parsed });
  // For security reasons, we won't get inline env variables.
  // Do not inject project process.env by default, cause it's unsafe
  prefixes = arraify(prefixes);
  for (const [key, value] of Object.entries(parsed)) {
    if (prefixes.some((prefix) => key.startsWith(prefix))) {
      env[key] = value;
    }
  }

  return [env, existsEnvFiles];
}

export type CompilationMode = "development" | "production";

export function setProcessEnv(mode: CompilationMode) {
  process.env.NODE_ENV = mode;
}
