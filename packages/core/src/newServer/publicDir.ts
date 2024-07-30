import { ResolvedUserConfig } from '../config/types.js';
import { recursiveReaddir } from '../utils/index.js';

export const ERR_SYMLINK_IN_RECURSIVE_READDIR =
  'ERR_SYMLINK_IN_RECURSIVE_READDIR';

const publicFilesMap = new WeakMap<ResolvedUserConfig, Set<string>>();

export async function initPublicFiles(
  config: ResolvedUserConfig
): Promise<Set<string> | undefined> {
  let fileNames: string[];
  const publicDir: string = config.publicDir;

  try {
    fileNames = await recursiveReaddir(publicDir);
  } catch (e) {
    if (e.code === ERR_SYMLINK_IN_RECURSIVE_READDIR) {
      return;
    }
    throw e;
  }
  const publicFiles = new Set(
    fileNames.map((fileName) => fileName.slice(publicDir.length))
  );
  publicFilesMap.set(config, publicFiles);
  return publicFiles;
}

export function getPublicFiles(
  config: ResolvedUserConfig
): Set<string> | undefined {
  return publicFilesMap.get(config);
}
