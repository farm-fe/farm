import { buildSsrApp, type SsrBuildOptions } from './build-preview.js';
import {
  createSsrServer,
  type SsrServer,
  type SsrServerOptions,
  startSsrServer
} from './server.js';

export type SsrToolkitCommand = 'dev' | 'preview' | 'build';

export type SsrRunServerCommandOptions = SsrServerOptions & {
  command?: 'dev' | 'preview';
  start?: boolean;
};

export type SsrRunBuildCommandOptions = SsrBuildOptions & {
  command: 'build';
};

export type SsrRunCommandOptions =
  | SsrRunServerCommandOptions
  | SsrRunBuildCommandOptions;

export type SsrRunCommandResult = void | SsrServer;

interface SsrRunCommandResolvers {
  buildSsrApp(options: SsrBuildOptions): Promise<void>;
  createSsrServer(options: SsrServerOptions): Promise<SsrServer>;
  startSsrServer(options: SsrServerOptions): Promise<SsrServer>;
}

const defaultResolvers: SsrRunCommandResolvers = {
  buildSsrApp,
  createSsrServer,
  startSsrServer
};

function isBuildCommand(
  options: SsrRunCommandOptions
): options is SsrRunBuildCommandOptions {
  return options.command === 'build';
}

function toServerOptions(
  options: SsrRunServerCommandOptions
): SsrServerOptions {
  const { start: _start, ...serverOptions } = options;
  return serverOptions as SsrServerOptions;
}

export async function runSsrCommandWithResolvers(
  options: SsrRunCommandOptions,
  resolvers: SsrRunCommandResolvers
): Promise<SsrRunCommandResult> {
  if (isBuildCommand(options)) {
    await resolvers.buildSsrApp(options);
    return;
  }

  const serverOptions = toServerOptions(options);
  const shouldStart = options.start ?? true;

  if (shouldStart) {
    return resolvers.startSsrServer(serverOptions);
  }

  return resolvers.createSsrServer(serverOptions);
}

export async function runSsrCommand(
  options: SsrRunCommandOptions
): Promise<SsrRunCommandResult> {
  return runSsrCommandWithResolvers(options, defaultResolvers);
}
