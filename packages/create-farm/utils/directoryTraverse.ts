import * as fs from 'node:fs';
import * as path from 'node:path';

export function preOrderDirectoryTraverse(
  dir: string,
  dirCallback: (arg: string) => void,
  fileCallback: (arg: string) => void
) {
  for (const filename of fs.readdirSync(dir)) {
    if (filename === '.git') {
      continue;
    }
    const fullpath = path.resolve(dir, filename);
    if (fs.lstatSync(fullpath).isDirectory()) {
      dirCallback(fullpath);
      // in case the dirCallback removes the directory entirely
      if (fs.existsSync(fullpath)) {
        preOrderDirectoryTraverse(fullpath, dirCallback, fileCallback);
      }
      continue;
    }
    fileCallback(fullpath);
  }
}

export function postOrderDirectoryTraverse(
  dir: string,
  dirCallback: (arg: string) => void,
  fileCallback: (arg: string) => void
) {
  for (const filename of fs.readdirSync(dir)) {
    if (filename === '.git') {
      continue;
    }
    const fullpath = path.resolve(dir, filename);
    if (fs.lstatSync(fullpath).isDirectory()) {
      postOrderDirectoryTraverse(fullpath, dirCallback, fileCallback);
      dirCallback(fullpath);
      continue;
    }
    fileCallback(fullpath);
  }
}
