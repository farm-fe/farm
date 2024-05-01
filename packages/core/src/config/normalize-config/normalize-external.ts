import module from 'node:module';

import { existsSync, readFileSync } from 'node:fs';
import path from 'node:path';
import type { Config } from '../../../binding/index.js';

export function normalizeExternal(config: Config['config']) {
  const defaultExternals: string[] = [];
  const externalNodeBuiltins = config.externalNodeBuiltins ?? true;

  if (externalNodeBuiltins) {
    if (Array.isArray(externalNodeBuiltins)) {
      defaultExternals.push(...externalNodeBuiltins);
    } else if (externalNodeBuiltins === true) {
      let packageJson: any = {};
      const pkgPath = path.join(config.root || process.cwd(), 'package.json');
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
            !config.resolve?.alias?.[m] &&
            !packageJson?.devDependencies?.[m] &&
            !packageJson?.dependencies?.[m]
        )
      );
    }
  }

  config.external = [
    ...(config.external ?? []),
    '^node:',
    ...defaultExternals.map((m) => `^${m}($|/promises$)`)
  ];
}
