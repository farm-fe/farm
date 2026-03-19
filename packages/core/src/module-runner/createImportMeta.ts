import type { FarmRunnerImportMeta } from './types.js';

const envProxy = new Proxy({} as Record<string, unknown>, {
  get(_, key) {
    throw new Error(
      `[farm module runner] Dynamic access of "import.meta.env" is not supported. Use "import.meta.env.${String(
        key
      )}" instead.`
    );
  }
});

export function createDefaultImportMeta(
  modulePath: string
): FarmRunnerImportMeta {
  const normalized = toFileUrl(modulePath);

  return {
    url: normalized,
    env: envProxy,
    dirname: dirname(modulePath),
    filename: modulePath,
    resolve() {
      throw new Error(
        '[farm module runner] "import.meta.resolve" is not supported.'
      );
    }
  } as FarmRunnerImportMeta;
}

function toFileUrl(modulePath: string): string {
  const normalized = modulePath.replace(/\\/g, '/');

  if (/^[a-zA-Z]:\//.test(normalized)) {
    return `file:///${encodeURI(normalized)}`;
  }

  if (/^[a-zA-Z][a-zA-Z\d+\-.]*:/.test(modulePath)) {
    return modulePath;
  }

  if (normalized.startsWith('//')) {
    return `file:${encodeURI(normalized)}`;
  }

  if (normalized.startsWith('/')) {
    return `file://${encodeURI(normalized)}`;
  }

  const cwd = getRuntimeCwd();

  if (!cwd) {
    return modulePath;
  }

  const baseUrl = toAbsoluteDirectoryFileUrl(cwd);

  if (!baseUrl) {
    return modulePath;
  }

  try {
    return new URL(normalized, baseUrl).toString();
  } catch {
    return modulePath;
  }
}

function getRuntimeCwd(): string | null {
  const processLike = globalThis.process as
    | {
        cwd?: () => string;
      }
    | undefined;

  if (!processLike || typeof processLike.cwd !== 'function') {
    return null;
  }

  try {
    const cwd = processLike.cwd();

    if (typeof cwd !== 'string' || !cwd) {
      return null;
    }

    return cwd.replace(/\\/g, '/');
  } catch {
    return null;
  }
}

function toAbsoluteDirectoryFileUrl(dirPath: string): string | null {
  const normalized = dirPath.replace(/\\/g, '/').replace(/\/+$/, '');

  if (/^[a-zA-Z]:\//.test(normalized)) {
    return `file:///${encodeURI(normalized)}/`;
  }

  if (normalized.startsWith('/')) {
    return `file://${encodeURI(normalized)}/`;
  }

  return null;
}

function dirname(modulePath: string): string {
  const normalized = modulePath.replace(/\\/g, '/').replace(/\/+$/, '');
  const index = normalized.lastIndexOf('/');

  if (index < 0) {
    return '.';
  }

  if (index === 0) {
    return normalized.slice(0, 1);
  }

  return normalized.slice(0, index);
}
