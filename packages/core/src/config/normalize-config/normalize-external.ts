import module from 'node:module';

import { Config } from '../../../binding/index.js';

export function normalizeExternal(config: Config['config']) {
  const defaultExternals = [...module.builtinModules, ...module.builtinModules]
    .filter((m) => !config.resolve?.alias?.[m])
    .map((m) => `^${m}$`);

  config.external = [
    ...(config.external ?? []),
    ...defaultExternals.map((m) => `^${m}($|/)`),
    ...defaultExternals.map((m) => `^node:${m}($|/)`)
  ];
}
