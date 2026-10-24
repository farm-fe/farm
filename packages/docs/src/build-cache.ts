import { createHash } from 'node:crypto';
import fs from 'node:fs/promises';
import type { ResolvedUserConfig } from '@farmfe/core';

function canonical(
  value: unknown,
  roles: ReadonlyMap<string, string>
): unknown {
  if (typeof value === 'string') return roles.get(value) ?? value;
  if (value === null || value === undefined || typeof value === 'boolean')
    return value;
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (Array.isArray(value)) return value.map((item) => canonical(item, roles));
  if (
    typeof value !== 'object' ||
    Object.getPrototypeOf(value) !== Object.prototype
  )
    throw new Error('[DOCS_CACHE_IDENTITY] configuration must be serializable');
  return Object.fromEntries(
    Object.entries(value)
      .sort(([a], [b]) => a.localeCompare(b))
      .filter(([, item]) => item !== undefined)
      .map(([key, item]) => [key, canonical(item, roles)])
  );
}

/** Only explicit transaction artifacts are relocatable, never arbitrary path substrings. */
export function docsCacheIdentity(
  value: unknown,
  roles: ReadonlyMap<string, string>
): string {
  const serialized = JSON.stringify(canonical(value, roles));
  if (serialized === undefined)
    throw new Error('[DOCS_CACHE_IDENTITY] identity is missing');
  return createHash('sha256').update(serialized).digest('hex');
}

export async function isolateDocsTargetCache(
  resolved: ResolvedUserConfig,
  options: {
    target: 'browser' | 'node';
    entryName: string;
    snapshotId: string;
    site: unknown;
    nativeIdentity: { path: string; sha256: string };
    roles: ReadonlyMap<string, string>;
    nativeHashes: Map<string, Promise<string>>;
  }
): Promise<void> {
  const compilation = resolved.compilation;
  const cache = compilation?.persistentCache;
  if (!cache || typeof cache !== 'object') return;
  const plugins = await Promise.all(
    (resolved.rustPlugins ?? []).map(async ([file, raw]) => {
      let pending = options.nativeHashes.get(file);
      if (!pending) {
        pending = fs
          .readFile(file)
          .then((bytes) => createHash('sha256').update(bytes).digest('hex'));
        options.nativeHashes.set(file, pending);
      }
      return { file, sha256: await pending, options: JSON.parse(raw) };
    })
  );
  const identity = docsCacheIdentity(
    {
      protocol: 'farm-docs-target-cache:1',
      node: process.versions,
      platform: process.platform,
      arch: process.arch,
      core: options.nativeIdentity,
      target: options.target,
      entryName: options.entryName,
      snapshotId: options.snapshotId,
      site: options.site,
      compilation,
      plugins
    },
    options.roles
  );
  cache.namespace = `${cache.namespace ?? 'farm-docs'}-${options.target}-${options.entryName}`;
  cache.envs = { ...cache.envs, FARM_DOCS_TARGET_IDENTITY: identity };
}
