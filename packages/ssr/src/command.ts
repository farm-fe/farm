import { buildSsrApp, type SsrBuildOptions } from './build-preview.js';
import {
  resolveSsrConfigForCommand,
  type SsrConfig
} from './config-resolver.js';
import {
  createSsrRuntime,
  type SsrRuntime,
  type SsrRuntimeConfig
} from './runtime.js';

export type SsrToolkitCommand = 'dev' | 'preview' | 'build';

export type SsrRunServerCommandOptions = SsrRuntimeConfig & {
  start?: boolean;
};

export type SsrRunBuildCommandOptions = SsrBuildOptions & {
  command: 'build';
};

export type SsrRunCommandOptions =
  | SsrRunServerCommandOptions
  | SsrRunBuildCommandOptions
  | (SsrConfig & SsrRunBuildCommandOptions);

export type SsrRunCommandResult = void | SsrRuntime;

interface SsrRunCommandResolvers {
  buildSsrApp(options: SsrBuildOptions): Promise<void>;
  createSsrRuntime(options: SsrRuntimeConfig): Promise<SsrRuntime>;
}

const defaultResolvers: SsrRunCommandResolvers = {
  buildSsrApp,
  createSsrRuntime
};

function isBuildCommand(
  options: SsrRunCommandOptions
): options is SsrRunBuildCommandOptions {
  return options.command === 'build';
}

export async function runSsrCommandWithResolvers(
  options: SsrRunCommandOptions,
  resolvers: SsrRunCommandResolvers
): Promise<SsrRunCommandResult> {
  if (isBuildCommand(options)) {
    const resolved = resolveSsrConfigForCommand(options as SsrConfig, 'build');
    await resolvers.buildSsrApp({
      ...(options as SsrBuildOptions),
      client: resolved.client,
      server: resolved.server ?? (options as SsrBuildOptions).server
    });
    return;
  }

  const { start: _start, ...runtimeOptions } =
    options as SsrRunServerCommandOptions;
  const shouldStart = options.start ?? true;
  const runtime = await resolvers.createSsrRuntime(
    runtimeOptions as SsrRuntimeConfig
  );

  if (shouldStart) {
    await runtime.start();
  }

  return runtime;
}

export async function runSsrCommand(
  options: SsrRunCommandOptions
): Promise<SsrRunCommandResult> {
  return runSsrCommandWithResolvers(options, defaultResolvers);
}
