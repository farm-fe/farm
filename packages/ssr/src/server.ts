import {
  createSsrPreviewServer,
  type SsrPreviewOptions,
  type SsrPreviewServer,
  startSsrPreviewServer
} from './build-preview.js';
import { resolveSsrConfigForCommand } from './config-resolver.js';
import {
  createSsrDevServer,
  type SsrDevServer,
  type SsrDevServerOptions,
  startSsrDevServer
} from './dev-server.js';

export type SsrServerCommand = 'dev' | 'preview';
export type SsrServerMode = SsrServerCommand;

type SsrServerCommonOptions = {
  command?: SsrServerCommand;
  // mode follows Farm config env semantics, not dispatch semantics.
  mode?: string;
};

type SsrServerDevOptions = SsrDevServerOptions &
  SsrServerCommonOptions & { command?: 'dev' };
type SsrServerPreviewOptions = SsrPreviewOptions &
  SsrServerCommonOptions & { command: 'preview' };

export type SsrServerOptions = SsrServerDevOptions | SsrServerPreviewOptions;

export type SsrServer = SsrDevServer | SsrPreviewServer;

interface SsrServerResolvers {
  createDevServer(options: SsrDevServerOptions): Promise<SsrDevServer>;
  createPreviewServer(options: SsrPreviewOptions): Promise<SsrPreviewServer>;
  startDevServer(options: SsrDevServerOptions): Promise<SsrDevServer>;
  startPreviewServer(options: SsrPreviewOptions): Promise<SsrPreviewServer>;
}

const defaultResolvers: SsrServerResolvers = {
  createDevServer: createSsrDevServer,
  createPreviewServer: createSsrPreviewServer,
  startDevServer: startSsrDevServer,
  startPreviewServer: startSsrPreviewServer
};

function resolveCommand(options: SsrServerOptions): SsrServerCommand {
  return options.command ?? 'dev';
}

function resolveMode(
  options: SsrServerOptions,
  command: SsrServerCommand
): string {
  if (options.mode) {
    return options.mode;
  }

  return command === 'preview' ? 'production' : 'development';
}

function toDevServerOptions(options: SsrServerOptions): SsrDevServerOptions {
  const command = resolveCommand(options);
  const mode = resolveMode(options, command);
  const resolved = resolveSsrConfigForCommand(options, command);

  return {
    ...resolved,
    client: {
      ...resolved.client,
      mode: resolved.client.mode ?? mode
    },
    ...(resolved.server
      ? {
          server: {
            ...resolved.server,
            mode: resolved.server.mode ?? mode
          }
        }
      : {})
  };
}

function toPreviewServerOptions(options: SsrServerOptions): SsrPreviewOptions {
  const command = resolveCommand(options);
  const mode = resolveMode(options, command);
  const resolved = resolveSsrConfigForCommand(options, command);

  return {
    ...resolved,
    client: {
      ...resolved.client,
      mode: resolved.client.mode ?? mode
    },
    server: {
      ...(resolved.server ?? {}),
      mode: resolved.server?.mode ?? mode
    }
  };
}

function isPreviewCommand(options: SsrServerOptions): boolean {
  return resolveCommand(options) === 'preview';
}

export async function createSsrServerWithResolvers(
  options: SsrServerOptions,
  resolvers: SsrServerResolvers
): Promise<SsrServer> {
  if (isPreviewCommand(options)) {
    return resolvers.createPreviewServer(toPreviewServerOptions(options));
  }

  return resolvers.createDevServer(toDevServerOptions(options));
}

export async function createSsrServer(
  options: SsrServerOptions
): Promise<SsrServer> {
  return createSsrServerWithResolvers(options, defaultResolvers);
}

export async function startSsrServerWithResolvers(
  options: SsrServerOptions,
  resolvers: SsrServerResolvers
): Promise<SsrServer> {
  if (isPreviewCommand(options)) {
    return resolvers.startPreviewServer(toPreviewServerOptions(options));
  }

  return resolvers.startDevServer(toDevServerOptions(options));
}

export async function startSsrServer(
  options: SsrServerOptions
): Promise<SsrServer> {
  return startSsrServerWithResolvers(options, defaultResolvers);
}
