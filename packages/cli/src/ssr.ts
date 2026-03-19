import { existsSync } from 'node:fs';
import path from 'node:path';
import type { FarmCliOptions, UserConfig } from '@farmfe/core';
import type { CliSsrOptions, GlobalCliOptions } from './types.js';
import { resolveRootPath } from './utils.js';

export type SsrCliCommand = 'dev' | 'build' | 'preview';

type SsrHostOptions = {
  hostname?: string;
  port?: number;
};

type SsrTemplateOptions = {
  file?: string;
  resource?: string;
  placeholder?: string;
};

type SsrRunOptions = {
  command: SsrCliCommand;
  mode?: string;
  client: FarmCliOptions & UserConfig;
  server: FarmCliOptions & UserConfig;
  host?: SsrHostOptions;
  ssr?: {
    entry: string;
    exportName?: string;
    template?: SsrTemplateOptions;
  };
};

const DEFAULT_DEV_ENTRY_CANDIDATES = [
  '/src/entry-server.mjs',
  '/src/entry-server.ts',
  '/src/entry-server.js',
  '/src/entry-server.tsx',
  '/src/entry-server.jsx'
] as const;

const DEFAULT_PREVIEW_ENTRY = 'index.js';
const DEFAULT_CLIENT_OUTPUT_PATH = 'dist/client';
const DEFAULT_SERVER_OUTPUT_PATH = 'dist/server';

function resolveConfigFile(params: {
  sharedConfigFile?: string;
  dedicatedConfigFile?: string;
  fallback: string;
}) {
  return (
    params.dedicatedConfigFile ?? params.sharedConfigFile ?? params.fallback
  );
}

function parsePort(rawPort: unknown): number | undefined {
  if (rawPort == null) {
    return undefined;
  }

  const parsed = Number(rawPort);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(
      `[farm ssr] invalid --port value "${String(rawPort)}", expected a positive number.`
    );
  }

  return parsed;
}

function resolveHost(options: CliSsrOptions): SsrHostOptions | undefined {
  const port = parsePort(options.port);
  const hostname = options.host;

  if (!hostname && port == null) {
    return undefined;
  }

  return {
    hostname,
    ...(port == null ? {} : { port })
  };
}

function resolveTemplate(
  command: SsrCliCommand,
  options: CliSsrOptions
): SsrTemplateOptions | undefined {
  if (command === 'build') {
    return undefined;
  }

  if (command === 'preview' && options.templateResource) {
    throw new Error(
      '[farm ssr] --template-resource is only supported by "farm ssr dev".'
    );
  }

  const placeholder = options.placeholder;

  if (command === 'preview') {
    if (!options.templateFile && !placeholder) {
      return undefined;
    }

    return {
      ...(options.templateFile ? { file: options.templateFile } : {}),
      ...(placeholder ? { placeholder } : {})
    };
  }

  if (options.templateFile) {
    return {
      file: options.templateFile,
      ...(placeholder ? { placeholder } : {})
    };
  }

  return {
    resource: options.templateResource ?? 'index.html',
    ...(placeholder ? { placeholder } : {})
  };
}

function toAbsolutePath(root: string, file: string) {
  if (path.isAbsolute(file)) {
    return file;
  }

  return path.join(root, file.replace(/^\//, ''));
}

function resolveEntry(params: {
  command: SsrCliCommand;
  root: string;
  options: GlobalCliOptions & CliSsrOptions;
}) {
  if (params.options.entry) {
    return params.options.entry;
  }

  const envEntry =
    (params.command === 'dev'
      ? process.env.FARM_SSR_DEV_ENTRY
      : process.env.FARM_SSR_PREVIEW_ENTRY) ?? process.env.FARM_SSR_ENTRY;

  if (envEntry) {
    return envEntry;
  }

  if (params.command === 'preview') {
    return DEFAULT_PREVIEW_ENTRY;
  }

  for (const candidate of DEFAULT_DEV_ENTRY_CANDIDATES) {
    if (existsSync(toAbsolutePath(params.root, candidate))) {
      return candidate;
    }
  }

  return DEFAULT_DEV_ENTRY_CANDIDATES[0];
}

function resolveBuildEntry(params: {
  root: string;
  options: GlobalCliOptions & CliSsrOptions;
}) {
  if (params.options.entry) {
    return params.options.entry;
  }

  if (process.env.FARM_SSR_ENTRY) {
    return process.env.FARM_SSR_ENTRY;
  }

  for (const candidate of DEFAULT_DEV_ENTRY_CANDIDATES) {
    if (existsSync(toAbsolutePath(params.root, candidate))) {
      return candidate;
    }
  }

  return DEFAULT_DEV_ENTRY_CANDIDATES[0];
}

function ensureCompilation(
  config: FarmCliOptions & UserConfig
): NonNullable<UserConfig['compilation']> {
  if (!config.compilation) {
    config.compilation = {};
  }
  return config.compilation;
}

function applyBuildDefaults(params: {
  root: string;
  options: GlobalCliOptions & CliSsrOptions;
  client: FarmCliOptions & UserConfig;
  server: FarmCliOptions & UserConfig;
}) {
  const clientCompilation = ensureCompilation(params.client);
  const serverCompilation = ensureCompilation(params.server);

  if (
    !clientCompilation.input ||
    Object.keys(clientCompilation.input).length === 0
  ) {
    clientCompilation.input = {
      index: './index.html'
    };
  }

  if (
    !serverCompilation.input ||
    Object.keys(serverCompilation.input).length === 0
  ) {
    serverCompilation.input = {
      index: resolveBuildEntry({
        root: params.root,
        options: params.options
      })
    };
  }

  clientCompilation.output = {
    path: DEFAULT_CLIENT_OUTPUT_PATH,
    ...(clientCompilation.output ?? {})
  };
  serverCompilation.output = {
    path: DEFAULT_SERVER_OUTPUT_PATH,
    targetEnv: 'node',
    format: 'esm',
    ...(serverCompilation.output ?? {})
  };
}

function buildCompilationOutput(options: GlobalCliOptions & CliSsrOptions) {
  const output: Record<string, unknown> = {};

  if (options.base) {
    output.publicPath = options.base;
  }
  if (options.outDir) {
    output.path = options.outDir;
  }
  if (options.target) {
    output.targetEnv = options.target;
  }
  if (options.format) {
    output.format = options.format;
  }

  if (Object.keys(output).length === 0) {
    return undefined;
  }

  return output;
}

function buildFarmConfig(params: {
  root: string;
  mode?: string;
  clearScreen?: boolean;
  configFile: string;
  options: GlobalCliOptions & CliSsrOptions;
}) {
  const output = buildCompilationOutput(params.options);
  const serverConfig: Record<string, unknown> = {};

  if (params.options.host) {
    serverConfig.host = params.options.host;
  }
  if (params.options.port != null) {
    serverConfig.port = parsePort(params.options.port);
  }
  if (params.options.open != null) {
    serverConfig.open = params.options.open;
  }
  if (params.options.strictPort != null) {
    serverConfig.strictPort = params.options.strictPort;
  }
  const hasPreviewServerOverride =
    params.options.host != null ||
    params.options.port != null ||
    params.options.open != null ||
    params.options.strictPort != null ||
    params.options.outDir != null;

  if (hasPreviewServerOverride) {
    serverConfig.preview = {
      ...(params.options.outDir ? { distDir: params.options.outDir } : {}),
      ...(params.options.host ? { host: params.options.host } : {}),
      ...(params.options.port != null
        ? { port: parsePort(params.options.port) }
        : {}),
      ...(params.options.open != null ? { open: params.options.open } : {}),
      ...(params.options.strictPort != null
        ? { strictPort: params.options.strictPort }
        : {})
    };
  }

  return {
    root: params.root,
    configFile: params.configFile,
    mode: params.mode,
    clearScreen: params.clearScreen,
    ...(Object.keys(serverConfig).length > 0 ? { server: serverConfig } : {}),
    ...(params.options.sourcemap != null ||
    params.options.minify != null ||
    params.options.treeShaking != null ||
    params.options.input != null ||
    output
      ? {
          compilation: {
            ...(output ? { output } : {}),
            ...(params.options.input
              ? {
                  input: {
                    index: params.options.input
                  }
                }
              : {}),
            ...(params.options.sourcemap != null
              ? { sourcemap: params.options.sourcemap }
              : {}),
            ...(params.options.minify != null
              ? { minify: params.options.minify }
              : {}),
            ...(params.options.treeShaking != null
              ? { treeShaking: params.options.treeShaking }
              : {})
          }
        }
      : {})
  } as FarmCliOptions & UserConfig;
}

export function resolveSsrRunOptions(params: {
  command: SsrCliCommand;
  root?: string;
  options: GlobalCliOptions & CliSsrOptions;
}): SsrRunOptions {
  const root = resolveRootPath(params.root);
  const clientConfigFile = resolveConfigFile({
    sharedConfigFile: params.options.config,
    dedicatedConfigFile: params.options.clientConfig,
    fallback: 'farm.config.client.ts'
  });
  const serverConfigFile = resolveConfigFile({
    sharedConfigFile: params.options.config,
    dedicatedConfigFile: params.options.serverConfig,
    fallback: 'farm.config.server.ts'
  });

  const resolved: SsrRunOptions = {
    command: params.command,
    mode: params.options.mode,
    client: buildFarmConfig({
      root,
      mode: params.options.mode,
      clearScreen: params.options.clearScreen,
      configFile: clientConfigFile,
      options: params.options
    }),
    server: buildFarmConfig({
      root,
      mode: params.options.mode,
      clearScreen: params.options.clearScreen,
      configFile: serverConfigFile,
      options: params.options
    }),
    host: resolveHost(params.options)
  };

  if (params.command === 'build') {
    applyBuildDefaults({
      root,
      options: params.options,
      client: resolved.client,
      server: resolved.server
    });
    return resolved;
  }

  const template = resolveTemplate(params.command, params.options);
  const entry = resolveEntry({
    command: params.command,
    root,
    options: params.options
  });

  resolved.ssr = {
    entry,
    ...(params.options.exportName
      ? { exportName: params.options.exportName }
      : {}),
    ...(template ? { template } : {})
  };

  return resolved;
}
