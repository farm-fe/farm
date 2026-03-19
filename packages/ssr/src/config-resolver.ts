import type { FarmCliOptions, UserConfig } from '@farmfe/core';

export type SsrConfig = {
  root?: string;
  mode?: string;
  command?: 'dev' | 'preview' | 'build';
  ssr?: unknown;
  hooks?: unknown;
  host?: unknown;
  runner?: unknown;
  client?: FarmCliOptions & UserConfig;
  server?: FarmCliOptions & UserConfig;
  $client?: FarmCliOptions & UserConfig;
  $server?: FarmCliOptions & UserConfig;
};

export function resolveSsrConfigForCommand<T extends SsrConfig>(
  config: T,
  command: 'dev' | 'preview' | 'build'
): {
  root?: string;
  mode?: string;
  ssr?: T['ssr'];
  hooks?: T['hooks'];
  host?: T['host'];
  runner?: T['runner'];
  client: FarmCliOptions & UserConfig;
  server?: FarmCliOptions & UserConfig;
} {
  const baseClient = config.client ?? ({} as FarmCliOptions & UserConfig);
  const baseServer = config.server ?? ({} as FarmCliOptions & UserConfig);
  const overrideClient = config.$client ?? ({} as FarmCliOptions & UserConfig);
  const overrideServer = config.$server ?? ({} as FarmCliOptions & UserConfig);

  const resolvedClient = {
    ...baseClient,
    ...overrideClient
  };
  const resolvedServer = config.server
    ? {
        ...baseServer,
        ...overrideServer
      }
    : config.$server
      ? {
          ...baseServer,
          ...overrideServer
        }
      : undefined;

  return {
    root: config.root,
    mode: config.mode,
    ssr: config.ssr,
    hooks: config.hooks,
    host: config.host,
    runner: config.runner,
    client: resolvedClient,
    server: resolvedServer
  };
}
