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

export function removeHashFromPath(path: string): string {
  const hashRE = /([_-][a-f0-9]{4,12})(\.[^./]+(\.[^./]+)*)$/;
  return path.replace(hashRE, '$2');
}
