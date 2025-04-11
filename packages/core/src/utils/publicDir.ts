import fs from 'fs';
import path from 'path';
import { UserConfig } from '../config/types.js';
import { ERR_SYMLINK_IN_RECURSIVE_READDIR, recursiveReaddir } from './file.js';
import { withTrailingSlash } from './path.js';
import { normalizePath } from './share.js';
import { cleanUrl } from './url.js';

const publicFilesMap = new WeakMap<UserConfig, Set<string>>();

export async function checkPublicFile(url: string, config: UserConfig) {
  // note if the file is in /public, the resolver would have returned it
  // as-is so it's not going to be a fully resolved path.
  const { publicDir } = config;
  if (!publicDir || url[0] !== '/') {
    return;
  }
  await initPublicFiles(config);
  const fileName = cleanUrl(url);
  const publicFiles = getPublicFiles(config);
  if (publicFiles) {
    return publicFiles.has(fileName)
      ? normalizePath(path.join(publicDir, fileName))
      : undefined;
  }

  const publicFile = normalizePath(path.join(publicDir, fileName));
  if (!publicFile.startsWith(withTrailingSlash(publicDir))) {
    // can happen if URL starts with '../'
    return;
  }
  return fs.existsSync(publicFile) ? publicFile : undefined;
}

export async function initPublicFiles(
  config: UserConfig
): Promise<Set<string> | undefined> {
  let fileNames: string[];
  try {
    fileNames = await recursiveReaddir(config.publicDir);
  } catch (e) {
    if (e.code === ERR_SYMLINK_IN_RECURSIVE_READDIR) {
      return;
    }
    throw e;
  }
  const publicFiles = new Set(
    fileNames.map((fileName) => fileName.slice(config.publicDir.length))
  );
  publicFilesMap.set(config, publicFiles);

  return publicFiles;
}

function getPublicFiles(config: UserConfig): Set<string> | undefined {
  return publicFilesMap.get(config);
}
