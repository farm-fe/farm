import { readFile } from 'node:fs/promises';

export async function tryReadFile(path: string) {
  try {
    return await readFile(path, { encoding: 'utf-8' });
  } catch (error) {
    console.error('[farm-solid-plugin]:\n' + error.message);
  }
}
