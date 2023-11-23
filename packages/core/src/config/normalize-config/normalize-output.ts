import module from 'node:module';

import { Config } from '../../../binding/index.js';

export async function normalizeOutput(
  config: Config['config'],
  isProduction: boolean
) {
  if (isProduction) {
    if (!config.output) {
      config.output = {};
    }
    if (!config.output.filename) {
      config.output.filename = '[resourceName].[contentHash].[ext]';
    }
    if (!config.output.assetsFilename) {
      config.output.assetsFilename = '[resourceName].[contentHash].[ext]';
    }
  }

  const defaultExternals = [...module.builtinModules, ...module.builtinModules]
    .filter((m) => !config.resolve?.alias?.[m])
    .map((m) => `^${m}$`);

  config.external = [
    ...(config.external ?? []),
    ...defaultExternals.map((m) => `^${m}($|/)`),
    ...defaultExternals.map((m) => `^node:${m}($|/)`)
  ];
}
