import path from 'node:path';
import type { UserConfig } from '@farmfe/core';
import { scaleProfile } from '../benchmark/scale-profiles.mjs';

export const groups = ['js', 'rust', 'mixed'] as const;
export type Group = (typeof groups)[number];
export function selectedGroup(): Group {
  const group = process.env.COMMERCE_GROUP ?? 'js';
  if (!groups.includes(group as Group))
    throw new Error(`Unknown COMMERCE_GROUP: ${group}`);
  return group as Group;
}

export function common(group: Group): UserConfig {
  const scale = process.env.COMMERCE_SCALE ?? 'baseline';
  scaleProfile(scale);
  const cacheId = process.env.COMMERCE_CACHE_ID ?? 'default';
  if (!/^[a-zA-Z0-9_-]+$/.test(cacheId)) throw new Error('Invalid COMMERCE_CACHE_ID');
  const outputId = process.env.COMMERCE_OUTPUT_ID;
  if (outputId !== undefined && !/^[a-zA-Z0-9_-]+$/.test(outputId)) throw new Error('Invalid COMMERCE_OUTPUT_ID');
  return {
    watch: { ignored: [/[\\/]\.cache([\\/]|$)/, /[\\/]dist([\\/]|$)/] },
    compilation: {
      resolve: { alias: { '@commerce-extension': path.resolve('packages/extensions', scale, 'index.ts') } },
      input: { index: './index.html' },
      output: { path: outputId ? `dist/benchmark/${outputId}/${group}` : `dist/${group}`, targetEnv: 'browser-esnext' },
      persistentCache: { cacheDir: path.resolve('.cache', group, cacheId) },
      lazyCompilation: false,
      sourcemap: true,
      minify: true,
      progress: false,
      presetEnv: false,
      define: { __COMMERCE_GROUP__: JSON.stringify(group) },
    },
    server: { port: Number(process.env.COMMERCE_PORT ?? 4310), preview: { port: Number(process.env.COMMERCE_PORT ?? 4310) }, open: false },
  };
}
