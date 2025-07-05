import { existsSync, readFileSync } from 'node:fs';
import module from 'node:module';
import path from 'node:path';

import { Config } from '../../types/binding.js';
import { isObject, safeJsonParse } from '../../utils/index.js';
import { CUSTOM_KEYS } from '../constants.js';

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
    config.compilation,
    mergeCustomExternal(
      resolvedCompilation,
      partialExternal(config.compilation.external)
    )
  );

  resolvedCompilation.custom[CUSTOM_KEYS.external_record] =
    JSON.stringify(recordExternal);

  resolvedCompilation.external = [
    ...stringExternal,
    '^node:',
    ...defaultExternals.map((m) => `^${m}($|/promises$)`)
  ];
}

export function mergeCustomExternal<
  T extends Partial<Pick<Config['config'], 'custom'>>
>(
  compilation: T,
  external: ReturnType<typeof partialExternal>
): PartialExternal {
  const [stringExternal, recordExternal] = external;
  if (!compilation?.custom) {
    compilation.custom = {};
  }

  const oldRecordExternal: Record<string, string> = compilation.custom[
    CUSTOM_KEYS.external_record
  ]
    ? safeJsonParse(compilation.custom[CUSTOM_KEYS.external_record], {}) || {}
    : {};

  return [
    [...new Set(stringExternal)],
    isObject(oldRecordExternal)
      ? { ...oldRecordExternal, ...recordExternal }
      : recordExternal
  ];
}
