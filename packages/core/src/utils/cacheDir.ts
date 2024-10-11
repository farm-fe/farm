import fs from 'fs';

export async function isCacheDirExists(dir: string): Promise<boolean> {
  try {
    const hasCacheDir = fs.readdirSync(dir, { withFileTypes: true });

    return !!(hasCacheDir && hasCacheDir.length);
  } catch (_) {
    return false;
  }
}
