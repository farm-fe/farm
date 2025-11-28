import { existsSync, readFileSync } from 'node:fs';
import module from 'node:module';
import path from 'node:path';

import { isObject } from '../../utils/index.js';

import type { ResolvedCompilation, UserConfig } from '../types.js';

type PartialExternal = [string[], Record<string, string>];

export function partialExternal(
  externalConfig: (string | Record<string, string>)[] = []
): PartialExternal {
  const stringExternal: string[] = [];
  const recordExternal: Record<string, string> = {};

  /**
   *
   * `["^node:.*$", { "jquery": "$" }]`
   * =>
   * `["^node:.*$"]`
   * `{ "jquery": "$" }`
   */
  for (const external of externalConfig) {
    if (typeof external === 'string') {
      stringExternal.push(external);
    } else if (isObject(external)) {
      Object.assign(recordExternal, external);
    }
  }

  return [stringExternal, recordExternal];
}

export function normalizeExternal(
  config: UserConfig,
  resolvedCompilation: ResolvedCompilation
) {
  const defaultExternals: string[] = [];
  const externalNodeBuiltins = config.compilation?.externalNodeBuiltins ?? true;

  if (externalNodeBuiltins) {
    if (Array.isArray(externalNodeBuiltins)) {
      defaultExternals.push(...externalNodeBuiltins);
    } else if (externalNodeBuiltins === true) {
      let packageJson: any = {};
      const pkgPath = path.join(
        resolvedCompilation.root || process.cwd(),
        'package.json'
      );
      // the project installed polyfill
      if (existsSync(pkgPath)) {
        try {
          packageJson = JSON.parse(readFileSync(pkgPath, 'utf8'));
        } catch {
          /**/
        }
      }

      defaultExternals.push(
        ...[...module.builtinModules].filter(
          (m) =>
            //@ts-ignore
            !resolvedCompilation.resolve?.alias?.[m] &&
            !packageJson?.devDependencies?.[m] &&
            !packageJson?.dependencies?.[m]
        )
      );
    }
  }

  if (!config?.compilation?.custom) {
    config.compilation ??= {};
    config.compilation.custom = {};
  }

  if (!resolvedCompilation?.custom) {
    resolvedCompilation.custom = {};
  }

  const [stringExternal, recordExternal] = mergeCustomExternal(
    partialExternal(config.compilation.external)
  );

  resolvedCompilation.output ??= {};
  resolvedCompilation.output.externalGlobals = recordExternal;

  resolvedCompilation.external = [
    ...stringExternal,
    '^node:',
    ...defaultExternals.map((m) => `^${m}($|/promises$)`)
  ];
}

export function mergeCustomExternal(
  external: ReturnType<typeof partialExternal>
): PartialExternal {
  const [stringExternal, recordExternal] = external;

  return [[...new Set(stringExternal)], recordExternal];
}
