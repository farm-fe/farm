const windowsSlashRE = /\\/g;
export function slash(p: string): string {
  return p.replace(windowsSlashRE, '/');
}

export function withTrailingSlash(path: string): string {
  if (path[path.length - 1] !== '/') {
    return `${path}/`;
  }
  return path;
}

const postfixRE = /[?#].*$/;
export function stripQueryAndHash(path: string): string {
  return path.replace(postfixRE, '');
}

export function removeHashFromPath(url: string): string {
  const hashPattern = /(_[a-zA-Z\d]{4,8})\./;

  const newURL = url.replace(hashPattern, '.');

  return newURL;
}
