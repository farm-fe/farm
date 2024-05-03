import module from 'node:module';

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import { ResolvedCompilation, UserConfig } from '../types.js';
import { isObject } from '../../utils/share.js';

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
            !resolvedCompilation.resolve?.alias?.[m] &&
            !packageJson?.devDependencies?.[m] &&
            !packageJson?.dependencies?.[m]
        )
      );
    }
  }

  const normalizedExternal: ResolvedCompilation['external'] = [];

  /**
   *
   * `["^node:.*$", { "jquery": "$" }]`
   * =>
   * `["^node:.*$", { pattern: "jquery", globalName: "$" }]`
   */
  for (const external of config?.compilation?.external ?? []) {
    if (typeof external === 'string') {
      normalizedExternal.push(external);
    } else if (isObject(external)) {
      for (const key in external) {
        if (!Object.hasOwn(external, key)) {
          continue;
        }

        normalizedExternal.push({
          pattern: key,
          globalName: external[key]
        });
      }
    }
  }

  resolvedCompilation.external = [
    ...normalizedExternal,
    '^node:',
    ...defaultExternals.map((m) => `^${m}($|/promises$)`)
  ];
}
