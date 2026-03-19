import { randomUUID } from 'node:crypto';
import fs from 'node:fs/promises';
import {
  createServer as createNodeServer,
  type IncomingMessage,
  type ServerResponse
} from 'node:http';
import path from 'node:path';
import {
  createCompiler,
  createModuleRunnerTransportFromInvokeHandlers,
  type FarmCliOptions,
  FarmModuleRunner,
  type FarmModuleRunnerOptions,
  type ModuleRunnerInvokeCompiler,
  normalizePath,
  type RunnerHotPayload,
  resolveConfig,
  Server,
  type UserConfig
} from '@farmfe/core';
import { createModuleRunnerInvokeHandlers } from '@farmfe/core/module-runner/serverInvoke';
import { toSsrError } from './errors.js';
import type { SsrRuntimeHooks } from './runtime-hooks.js';
import type {
  SsrRuntimeAssets,
  SsrRuntimeCommand,
  SsrRuntimeMeta
} from './runtime-types.js';

type SsrUpdateType = 'added' | 'updated' | 'removed';

type SsrUpdateItem = {
  path: string;
  type: SsrUpdateType;
};

type SsrUpdateResult = {
  added: string[];
  changed: string[];
  removed: string[];
  extraWatchResult: {
    add: string[];
  };
};

export interface SsrDevServerListenOptions {
  port?: number;
  hostname?: string;
}

export interface SsrDevServerOptions {
  client: FarmCliOptions & UserConfig;
  server?: FarmCliOptions & UserConfig;
  host?: SsrDevServerListenOptions;
  ssrMiddleware?: SsrNextMiddleware;
  ssr?: SsrRenderOptions;
  hooks?: SsrRuntimeHooks;
  runner?: Omit<FarmModuleRunnerOptions, 'transport'>;
}

export interface SsrDevServer {
  middlewares: SsrMiddlewareServer;
  runner: FarmModuleRunner;
  render(
    url: string,
    req: IncomingMessage,
    res: ServerResponse
  ): Promise<string>;
  listen(options?: SsrDevServerListenOptions): Promise<void>;
  close(): Promise<void>;
}

export type SsrMiddleware = (
  req: IncomingMessage,
  res: ServerResponse,
  next?: (error?: unknown) => void
) => void;

export type SsrNextMiddleware = (
  req: IncomingMessage,
  res: ServerResponse,
  next: (error?: unknown) => void
) => void;

export type SsrMiddlewareServer = SsrMiddleware & {
  use(middleware: SsrMiddleware): void;
};

export interface SsrTemplateLoadContext {
  url: string;
  req: IncomingMessage;
  res: ServerResponse;
  root: string;
}

export interface SsrTemplateRenderContext extends SsrTemplateLoadContext {
  appHtml: string;
  template: string;
}

export interface SsrTemplateOptions {
  resource?: string;
  file?: string;
  placeholder?: string;
  load?: (context: SsrTemplateLoadContext) => string | Promise<string>;
  transform?: (context: SsrTemplateRenderContext) => string | Promise<string>;
}

export interface SsrRenderContext {
  url: string;
  req: IncomingMessage;
  res: ServerResponse;
  command: SsrRuntimeCommand;
  mode: string;
  runtime: SsrRuntimeMeta;
  assets: SsrRuntimeAssets;
  runner?: FarmModuleRunner;
  requestId?: string;
}

export interface SsrRenderOptions {
  entry: string;
  exportName?: string;
  template: SsrTemplateOptions;
  shouldHandle?: (ctx: SsrRenderContext) => boolean;
  render?: (mod: Record<string, unknown>, ctx: SsrRenderContext) => unknown;
  onError?: (
    error: unknown,
    ctx: SsrRenderContext & { next: (error?: unknown) => void }
  ) => void | Promise<void>;
}

interface SsrTemplateResourceReader {
  resource: (name: string) => Buffer | string | undefined | null;
}

export interface SsrDevWatcherLike {
  on(
    event: 'add' | 'change' | 'unlink',
    listener: (path: string) => Promise<void> | void
  ): unknown;
  add(paths: string | string[]): unknown;
  filterWatchFile(file: string, root: string): boolean;
}

export interface SsrDevFarmServerLike {
  middlewares: SsrMiddleware;
  root?: string;
  getCompiler?: () => SsrTemplateResourceReader | undefined | null;
  watcher?: SsrDevWatcherLike;
  createModuleRunner(
    options?: Omit<FarmModuleRunnerOptions, 'transport'>
  ): Promise<FarmModuleRunner>;
  close(): Promise<void>;
}

export interface SsrDevHostServerLike {
  listening: boolean;
  listen(port: number, hostname?: string): SsrDevHostServerLike;
  close(
    callback?: (error?: Error | undefined) => void
  ): SsrDevHostServerLike | void;
  once(
    event: 'listening' | 'error',
    listener: (...args: unknown[]) => void
  ): this;
  off(
    event: 'listening' | 'error',
    listener: (...args: unknown[]) => void
  ): this;
}

export interface SsrDevCompilerLike extends ModuleRunnerInvokeCompiler {
  compile(): Promise<void>;
  update(paths: SsrUpdateItem[]): Promise<SsrUpdateResult>;
  hasModule(path: string): boolean;
  transformModulePath(root: string, path: string): string;
  resolvedModulePaths?(root: string): string[];
  invalidateModule?(moduleId: string): void;
}

export interface SsrDevServerCompilerCreateResult {
  root: string;
  publicPath: string;
  compiler: SsrDevCompilerLike;
}

export interface SsrDevServerFactories {
  createFarmServer(
    config: FarmCliOptions & UserConfig
  ): Promise<SsrDevFarmServerLike>;
  createHostServer(middlewares: SsrMiddleware): SsrDevHostServerLike;
  createServerCompiler(
    config: FarmCliOptions & UserConfig
  ): Promise<SsrDevServerCompilerCreateResult>;
}

const DEFAULT_HMR_PORT_RETRY_BASE = 9811;
const DEFAULT_HMR_PORT_RETRY_MAX_ATTEMPTS = 30;

const defaultFactories: SsrDevServerFactories = {
  createFarmServer(config) {
    return Server.createServer(config);
  },
  createHostServer(middlewares) {
    return createNodeServer(middlewares) as SsrDevHostServerLike;
  },
  async createServerCompiler(config) {
    const resolved = await resolveConfig(config, 'dev', 'development');
    const compiler = createCompiler(resolved) as unknown as SsrDevCompilerLike;
    await compiler.compile();
    return {
      root: resolved.root,
      publicPath: resolved.compilation.output.publicPath ?? '/',
      compiler
    };
  }
};

function isHtmlRequest(req: IncomingMessage) {
  const method = req.method?.toUpperCase();
  if (method !== 'GET' && method !== 'HEAD') {
    return false;
  }

  const url = req.url ?? '/';
  if (url.includes('/@') || url.includes('/__')) {
    return false;
  }
  const queryIndex = url.indexOf('?');
  const pathname = queryIndex >= 0 ? url.slice(0, queryIndex) : url;
  if (/\.[a-zA-Z0-9]+$/.test(pathname)) {
    return false;
  }

  const accept = req.headers.accept;
  if (!accept) {
    return true;
  }

  return accept.includes('text/html') || accept.includes('*/*');
}

function createMiddlewareServer() {
  const stack: SsrMiddleware[] = [];

  const middlewareServer = ((
    req: IncomingMessage,
    res: ServerResponse,
    next?: (error?: unknown) => void
  ) => {
    let index = 0;

    const run = (error?: unknown) => {
      if (error) {
        next?.(error);
        return;
      }

      const middleware = stack[index++];
      if (!middleware) {
        next?.();
        return;
      }

      try {
        middleware(req, res, run);
      } catch (err) {
        run(err);
      }
    };

    run();
  }) as SsrMiddlewareServer;

  middlewareServer.use = (middleware) => {
    stack.push(middleware);
  };

  return middlewareServer;
}

function resolveDefaultListenOptions(
  options: SsrDevServerOptions
): Required<SsrDevServerListenOptions> {
  const clientServer = options.client.server ?? {};
  const fallbackHost =
    typeof clientServer.host === 'string' ? clientServer.host : '127.0.0.1';

  return {
    port: options.host?.port ?? clientServer.port ?? 3000,
    hostname: options.host?.hostname ?? fallbackHost
  };
}

function resolveClientConfig(
  clientConfig: FarmCliOptions & UserConfig
): FarmCliOptions & UserConfig {
  const resolvedServerConfig = {
    ...(clientConfig.server ?? {}),
    middlewareMode: true
  };

  return {
    ...clientConfig,
    server: resolvedServerConfig
  };
}

function isHmrPortInUseError(error: unknown): boolean {
  if (!(error instanceof Error)) {
    return false;
  }

  const message = error.stack ?? error.message;
  return (
    message.includes('WebSocket server error: Port is already in use') ||
    message.includes('EADDRINUSE')
  );
}

function resolveHmrRetryBasePort(config: FarmCliOptions & UserConfig): number {
  const userServerConfig = config.server;

  if (
    userServerConfig?.hmr &&
    typeof userServerConfig.hmr === 'object' &&
    typeof userServerConfig.hmr.port === 'number' &&
    Number.isInteger(userServerConfig.hmr.port) &&
    userServerConfig.hmr.port > 0
  ) {
    return userServerConfig.hmr.port;
  }

  if (
    typeof userServerConfig?.port === 'number' &&
    Number.isInteger(userServerConfig.port) &&
    userServerConfig.port > 0
  ) {
    return userServerConfig.port;
  }

  return DEFAULT_HMR_PORT_RETRY_BASE;
}

function applyHmrPortToClientConfig(
  config: FarmCliOptions & UserConfig,
  port: number
): FarmCliOptions & UserConfig {
  const userServerConfig = config.server ?? {};

  if (userServerConfig.hmr === false) {
    return config;
  }

  const normalizedHmr =
    userServerConfig.hmr && typeof userServerConfig.hmr === 'object'
      ? userServerConfig.hmr
      : {};

  return {
    ...config,
    server: {
      ...userServerConfig,
      hmr: {
        ...normalizedHmr,
        port
      }
    }
  };
}

async function createFarmServerWithHmrPortRetry(params: {
  factories: SsrDevServerFactories;
  clientConfig: FarmCliOptions & UserConfig;
}): Promise<{
  farmServer: SsrDevFarmServerLike;
  resolvedClientConfig: FarmCliOptions & UserConfig;
}> {
  const initialConfig = resolveClientConfig(params.clientConfig);
  const serverConfig = initialConfig.server;

  if (serverConfig?.hmr === false) {
    const farmServer = await params.factories.createFarmServer(initialConfig);
    return {
      farmServer,
      resolvedClientConfig: initialConfig
    };
  }

  const basePort = resolveHmrRetryBasePort(initialConfig);
  let lastError: unknown;

  for (
    let attempt = 0;
    attempt < DEFAULT_HMR_PORT_RETRY_MAX_ATTEMPTS;
    attempt++
  ) {
    const candidatePort = basePort + attempt;
    const candidateConfig = applyHmrPortToClientConfig(
      initialConfig,
      candidatePort
    );

    try {
      const farmServer =
        await params.factories.createFarmServer(candidateConfig);
      return {
        farmServer,
        resolvedClientConfig: candidateConfig
      };
    } catch (error) {
      if (!isHmrPortInUseError(error)) {
        throw error;
      }
      lastError = error;
    }
  }

  const message =
    lastError instanceof Error ? lastError.message : String(lastError);
  throw new Error(
    `[farm ssr] failed to allocate an available HMR port after ${DEFAULT_HMR_PORT_RETRY_MAX_ATTEMPTS} attempts from ${basePort}. lastError=${message}`
  );
}

function listenHostServer(
  hostServer: SsrDevHostServerLike,
  options: Required<SsrDevServerListenOptions>
) {
  return new Promise<void>((resolve, reject) => {
    const onListening = () => {
      hostServer.off('error', onError);
      resolve();
    };
    const onError = (error: unknown) => {
      hostServer.off('listening', onListening);
      reject(error);
    };

    hostServer.once('listening', onListening);
    hostServer.once('error', onError);
    hostServer.listen(options.port, options.hostname);
  });
}

function closeHostServer(hostServer: SsrDevHostServerLike) {
  if (!hostServer.listening) {
    return Promise.resolve();
  }

  return new Promise<void>((resolve, reject) => {
    hostServer.close((error) => {
      if (error) {
        reject(error);
        return;
      }
      resolve();
    });
  });
}

function createHotBus() {
  const listeners = new Set<(payload: RunnerHotPayload) => void>();

  return {
    emit(payload: RunnerHotPayload) {
      for (const listener of listeners) {
        listener(payload);
      }
    },
    hotBus: {
      subscribe(cb: (payload: RunnerHotPayload) => void) {
        listeners.add(cb);
        return () => listeners.delete(cb);
      }
    }
  };
}

function emitRunnerPayload(
  emit: (payload: RunnerHotPayload) => void,
  result: SsrUpdateResult
) {
  const changedModules = [
    ...result.changed,
    ...result.added,
    ...result.extraWatchResult.add
  ];

  if (changedModules.length > 0) {
    emit({
      type: 'update',
      updates: changedModules.map((path) => ({
        type: 'js-update',
        path,
        acceptedPath: path,
        timestamp: Date.now()
      }))
    });
    return;
  }

  if (result.removed.length > 0) {
    emit({ type: 'full-reload' });
  }
}

function normalizeServerUpdateResultPaths(
  result: SsrUpdateResult,
  normalizePathByCompiler: (path: string) => string
): SsrUpdateResult {
  return {
    added: result.added.map(normalizePathByCompiler),
    changed: result.changed.map(normalizePathByCompiler),
    removed: result.removed.map(normalizePathByCompiler),
    extraWatchResult: {
      add: result.extraWatchResult.add.map(normalizePathByCompiler)
    }
  };
}

function watchServerCompilerWithSingleWatcher(params: {
  watcher: SsrDevWatcherLike;
  compiler: SsrDevCompilerLike;
  onUpdateResult: (
    result: SsrUpdateResult,
    updateError?: unknown,
    updateDurationMs?: number
  ) => void | Promise<void>;
}) {
  let active = true;

  const runUpdate = async (path: string, type: SsrUpdateType) => {
    if (!active) {
      return;
    }

    let result: SsrUpdateResult;
    let updateError: unknown;
    const startTime = Date.now();
    try {
      result = await params.compiler.update([{ path, type }]);
    } catch (error) {
      updateError = error;
      result = {
        added: [],
        changed: type === 'removed' ? [] : [path],
        removed: type === 'removed' ? [path] : [],
        extraWatchResult: {
          add: []
        }
      };
    }

    if (!active) {
      return;
    }

    await params.onUpdateResult(result, updateError, Date.now() - startTime);
  };

  params.watcher.on('add', async (file) => {
    await runUpdate(normalizePath(file), 'added');
  });

  params.watcher.on('unlink', async (file) => {
    await runUpdate(normalizePath(file), 'removed');
  });

  params.watcher.on('change', async (file) => {
    const normalizedFile = normalizePath(file);
    await runUpdate(normalizedFile, 'updated');
  });

  return () => {
    active = false;
  };
}

async function closeWithOrder(params: {
  hostServer: SsrDevHostServerLike;
  runner: FarmModuleRunner;
  farmServer: SsrDevFarmServerLike;
  stopWatcherSync: () => void;
}) {
  params.stopWatcherSync();

  const results = await Promise.allSettled([
    closeHostServer(params.hostServer),
    params.runner.close(),
    params.farmServer.close()
  ]);
  const rejected = results.find(
    (result): result is PromiseRejectedResult => result.status === 'rejected'
  );

  if (rejected) {
    throw rejected.reason;
  }
}

function normalizeTemplateResourcePath(resource: string) {
  if (resource.startsWith('/')) {
    return resource.slice(1);
  }

  return resource;
}

const DEFAULT_TEMPLATE_PLACEHOLDER = '<!--app-html-->';
const ROOT_CONTAINER_PATTERN =
  /<div\s+id=(?:"root"|'root'|root)[^>]*>[\s\S]*?<\/div>/;

function injectAppHtmlToTemplate(params: {
  template: string;
  appHtml: string;
  placeholder: string;
}) {
  if (params.template.includes(params.placeholder)) {
    return params.template.replace(params.placeholder, params.appHtml);
  }

  if (params.placeholder !== DEFAULT_TEMPLATE_PLACEHOLDER) {
    throw new Error(
      `[farm ssr] template placeholder "${params.placeholder}" was not found.`
    );
  }

  if (!ROOT_CONTAINER_PATTERN.test(params.template)) {
    throw new Error(
      `[farm ssr] template placeholder "${params.placeholder}" was not found.`
    );
  }

  return params.template.replace(
    ROOT_CONTAINER_PATTERN,
    `<div id="root">${params.appHtml}</div>`
  );
}

async function loadTemplateContent(params: {
  options: SsrRenderOptions;
  farmServer: SsrDevFarmServerLike;
  root: string;
  url: string;
  req: IncomingMessage;
  res: ServerResponse;
}) {
  const template = params.options.template;
  const loadContext: SsrTemplateLoadContext = {
    url: params.url,
    req: params.req,
    res: params.res,
    root: params.root
  };

  if (template.load) {
    return await template.load(loadContext);
  }

  if (template.file) {
    const filePath = path.isAbsolute(template.file)
      ? template.file
      : path.join(params.root, template.file);
    return await fs.readFile(filePath, 'utf-8');
  }

  if (template.resource) {
    const compiler = params.farmServer.getCompiler?.();
    const resource =
      compiler?.resource(template.resource) ??
      compiler?.resource(normalizeTemplateResourcePath(template.resource));

    if (resource == null) {
      throw new Error(
        `[farm ssr] template resource "${template.resource}" is not available in client compiler resources.`
      );
    }

    return Buffer.isBuffer(resource)
      ? resource.toString('utf-8')
      : String(resource);
  }

  throw new Error(
    '[farm ssr] invalid ssr.template configuration, provide one of template.load/template.file/template.resource.'
  );
}

function resolveRenderResult(params: {
  options: SsrRenderOptions;
  moduleExports: Record<string, unknown>;
  context: SsrRenderContext;
}) {
  if (params.options.render) {
    return params.options.render(params.moduleExports, params.context);
  }

  const exportName = params.options.exportName ?? 'default';
  const render = params.moduleExports[exportName];

  if (typeof render !== 'function') {
    throw new Error(
      `[farm ssr] export "${exportName}" from "${params.options.entry}" must be a function or provide ssr.render().`
    );
  }

  return (render as (url: string, ctx: unknown) => unknown)(
    params.context.url,
    params.context
  );
}

export async function renderSsrHtmlResponse(params: {
  options: SsrRenderOptions;
  farmServer: SsrDevFarmServerLike;
  root: string;
  context: SsrRenderContext;
}) {
  if (!params.context.runner) {
    throw new Error('[farm ssr] runner is required to render HTML in dev.');
  }
  const mod = (await params.context.runner.import(
    params.options.entry
  )) as Record<string, unknown>;

  const renderResult = await resolveRenderResult({
    options: params.options,
    moduleExports: mod,
    context: params.context
  });
  const appHtml =
    typeof renderResult === 'string' ? renderResult : String(renderResult);
  const template = await loadTemplateContent({
    options: params.options,
    farmServer: params.farmServer,
    root: params.root,
    url: params.context.url,
    req: params.context.req,
    res: params.context.res
  });

  if (params.options.template.transform) {
    return await params.options.template.transform({
      url: params.context.url,
      req: params.context.req,
      res: params.context.res,
      root: params.root,
      appHtml,
      template
    });
  }

  const placeholder =
    params.options.template.placeholder ?? DEFAULT_TEMPLATE_PLACEHOLDER;
  return injectAppHtmlToTemplate({
    template,
    appHtml,
    placeholder
  });
}

function createSsrRenderMiddleware(params: {
  options: SsrRenderOptions;
  root: string;
  farmServer: SsrDevFarmServerLike;
  runner: FarmModuleRunner;
  runtime: SsrRuntimeMeta;
  mode: string;
  assets?: SsrRuntimeAssets;
  hooks?: SsrRuntimeHooks;
}) {
  return async (
    req: IncomingMessage,
    res: ServerResponse,
    next: (error?: unknown) => void
  ) => {
    const requestId = randomUUID();
    const context: SsrRenderContext = {
      url: req.url ?? '/',
      req,
      res,
      runner: params.runner,
      command: 'dev',
      mode: params.mode,
      runtime: params.runtime,
      assets: params.assets ?? { css: [], preload: [], scripts: [] },
      requestId
    };
    const shouldHandle =
      params.options.shouldHandle ?? ((ctx) => isHtmlRequest(ctx.req));

    if (!shouldHandle(context)) {
      next();
      return;
    }

    const startTime = Date.now();
    try {
      params.hooks?.onRenderStart?.({
        requestId,
        url: context.url,
        command: context.command,
        mode: context.mode,
        runtime: context.runtime
      });
      const html = await renderSsrHtmlResponse({
        options: params.options,
        farmServer: params.farmServer,
        root: params.root,
        context
      });

      res.statusCode = 200;
      res.setHeader('Content-Type', 'text/html; charset=utf-8');
      res.end(html);
      params.hooks?.onRenderEnd?.({
        requestId,
        url: context.url,
        command: context.command,
        mode: context.mode,
        runtime: context.runtime,
        ms: Date.now() - startTime
      });
    } catch (error) {
      const ssrError = toSsrError({
        code: 'SSR_RENDER_FAILED',
        error,
        debug: params.mode === 'development'
      });
      params.hooks?.onRenderEnd?.({
        requestId,
        url: context.url,
        command: context.command,
        mode: context.mode,
        runtime: context.runtime,
        ms: Date.now() - startTime,
        error: ssrError
      });
      if (params.options.onError) {
        await params.options.onError(error, { ...context, next });
        return;
      }

      const stack =
        error instanceof Error ? (error.stack ?? error.message) : String(error);
      res.statusCode = 500;
      res.setHeader('Content-Type', 'text/plain; charset=utf-8');
      res.end(stack);
    }
  };
}

export async function createSsrDevServerWithFactories(
  options: SsrDevServerOptions,
  factories: SsrDevServerFactories
): Promise<SsrDevServer> {
  if (options.ssr && options.ssrMiddleware) {
    throw new Error(
      '[farm ssr] "ssr" and "ssrMiddleware" cannot be used together.'
    );
  }

  const { farmServer, resolvedClientConfig } =
    await createFarmServerWithHmrPortRetry({
      factories,
      clientConfig: options.client
    });
  const clientRoot = farmServer.root ?? options.client.root ?? process.cwd();
  let stopWatcherSync: () => void = () => {};

  let runner: FarmModuleRunner | null = null;

  try {
    if (!options.server) {
      runner = await farmServer.createModuleRunner({
        hmr: true,
        ...(options.runner ?? {})
      });
    } else {
      const serverCompilerResult = await factories.createServerCompiler(
        options.server
      );
      const invokeContext = {
        root: serverCompilerResult.root,
        publicPath: serverCompilerResult.publicPath,
        moduleRunnerStamp: 0,
        compiler: serverCompilerResult.compiler
      };
      const { emit, hotBus } = createHotBus();

      const invokeHandlers = createModuleRunnerInvokeHandlers(invokeContext);
      const transport = createModuleRunnerTransportFromInvokeHandlers({
        invokeHandlers,
        hotBus
      });
      runner = new FarmModuleRunner({
        ...options.runner,
        hmr: true,
        transport
      });

      if (!farmServer.watcher) {
        throw new Error(
          '[farm ssr] client dev watcher is required when server compiler is enabled.'
        );
      }

      const initialServerWatchFiles =
        serverCompilerResult.compiler.resolvedModulePaths?.(
          serverCompilerResult.root
        ) ?? [];
      const filteredInitialServerWatchFiles = initialServerWatchFiles.filter(
        (file) => farmServer.watcher.filterWatchFile(file, clientRoot)
      );
      if (filteredInitialServerWatchFiles.length > 0) {
        farmServer.watcher.add(filteredInitialServerWatchFiles);
      }

      stopWatcherSync = watchServerCompilerWithSingleWatcher({
        watcher: farmServer.watcher,
        compiler: serverCompilerResult.compiler,
        async onUpdateResult(result, updateError, updateDurationMs) {
          const changedCandidates = [
            ...result.changed,
            ...result.added,
            ...result.removed,
            ...result.extraWatchResult.add
          ];
          const hasServerImpact = changedCandidates.some((modulePath) =>
            invokeContext.compiler.hasModule(modulePath)
          );

          if (!hasServerImpact) {
            return;
          }

          options.hooks?.onInvalidate?.({
            kind: 'server',
            reason: updateError ? 'rebuild' : 'update',
            added: result.added.length + result.extraWatchResult.add.length,
            changed: result.changed.length,
            removed: result.removed.length,
            ms: updateDurationMs
          });

          if (updateError) {
            const recreatedCompilerResult =
              await factories.createServerCompiler(options.server);
            invokeContext.root = recreatedCompilerResult.root;
            invokeContext.publicPath = recreatedCompilerResult.publicPath;
            invokeContext.compiler = recreatedCompilerResult.compiler;

            const normalizePathByCompiler = (modulePath: string) =>
              recreatedCompilerResult.compiler.transformModulePath(
                recreatedCompilerResult.root,
                modulePath
              );
            const normalizedResult = normalizeServerUpdateResultPaths(
              result,
              normalizePathByCompiler
            );

            invokeContext.moduleRunnerStamp++;
            emit({ type: 'full-reload' });
            emitRunnerPayload(emit, normalizedResult);

            const addedWatchFiles = [
              ...normalizedResult.added,
              ...normalizedResult.extraWatchResult.add
            ];
            const recreatedWatchFiles =
              recreatedCompilerResult.compiler.resolvedModulePaths?.(
                recreatedCompilerResult.root
              ) ?? [];
            const mergedWatchFiles = [
              ...addedWatchFiles,
              ...recreatedWatchFiles
            ].filter((file) =>
              farmServer.watcher.filterWatchFile(file, clientRoot)
            );

            if (mergedWatchFiles.length > 0) {
              farmServer.watcher.add(mergedWatchFiles);
            }
            return;
          }

          const normalizePathByCompiler = (modulePath: string) =>
            invokeContext.compiler.transformModulePath(
              invokeContext.root,
              modulePath
            );
          const normalizedResult = normalizeServerUpdateResultPaths(
            result,
            normalizePathByCompiler
          );

          invokeContext.moduleRunnerStamp++;
          emitRunnerPayload(emit, normalizedResult);

          const addedWatchFiles = [
            ...normalizedResult.added,
            ...normalizedResult.extraWatchResult.add
          ].filter((file) =>
            farmServer.watcher.filterWatchFile(file, clientRoot)
          );

          if (addedWatchFiles.length > 0) {
            farmServer.watcher.add(addedWatchFiles);
          }
        }
      });
    }
  } catch (error) {
    await runner?.close().catch(() => undefined);
    await farmServer.close().catch(() => undefined);
    throw error;
  }

  if (!runner) {
    throw new Error('[farm ssr] runner initialization failed.');
  }

  const middlewares = createMiddlewareServer();
  const runtimeMeta: SsrRuntimeMeta = {
    root: clientRoot,
    publicPath: resolvedClientConfig.compilation?.output?.publicPath ?? '/'
  };
  const runtimeMode = resolvedClientConfig.mode ?? 'development';

  if (options.ssrMiddleware) {
    middlewares.use(options.ssrMiddleware);
  } else if (options.ssr) {
    middlewares.use(
      createSsrRenderMiddleware({
        options: options.ssr,
        root: clientRoot,
        farmServer,
        runner,
        runtime: runtimeMeta,
        mode: runtimeMode,
        hooks: options.hooks
      })
    );
  }

  middlewares.use(farmServer.middlewares);

  const hostServer = factories.createHostServer(middlewares);
  const defaultListenOptions = resolveDefaultListenOptions({
    ...options,
    client: resolvedClientConfig
  });
  let closed = false;

  return {
    middlewares,
    runner,
    async render(url, req, res) {
      if (!options.ssr) {
        throw new Error(
          '[farm ssr] ssr options are required to call render() in dev.'
        );
      }
      const requestId = randomUUID();
      const startTime = Date.now();
      options.hooks?.onRenderStart?.({
        requestId,
        url,
        command: 'dev',
        mode: runtimeMode,
        runtime: runtimeMeta
      });
      return renderSsrHtmlResponse({
        options: options.ssr,
        farmServer,
        root: clientRoot,
        context: {
          url,
          req,
          res,
          runner,
          command: 'dev',
          mode: runtimeMode,
          runtime: runtimeMeta,
          assets: { css: [], preload: [], scripts: [] },
          requestId
        }
      })
        .then((html) => {
          options.hooks?.onRenderEnd?.({
            requestId,
            url,
            command: 'dev',
            mode: runtimeMode,
            runtime: runtimeMeta,
            ms: Date.now() - startTime
          });
          return html;
        })
        .catch((error) => {
          const ssrError = toSsrError({
            code: 'SSR_RENDER_FAILED',
            error,
            debug: runtimeMode === 'development'
          });
          options.hooks?.onRenderEnd?.({
            requestId,
            url,
            command: 'dev',
            mode: runtimeMode,
            runtime: runtimeMeta,
            ms: Date.now() - startTime,
            error: ssrError
          });
          throw error;
        });
    },
    async listen(listenOptions) {
      if (closed) {
        throw new Error('[farm ssr] dev server is already closed.');
      }

      if (hostServer.listening) {
        return;
      }

      await listenHostServer(hostServer, {
        ...defaultListenOptions,
        ...(listenOptions ?? {})
      });
    },
    async close() {
      if (closed) {
        return;
      }
      closed = true;
      await closeWithOrder({
        hostServer,
        runner,
        farmServer,
        stopWatcherSync
      });
    }
  };
}

export async function createSsrDevServer(
  options: SsrDevServerOptions
): Promise<SsrDevServer> {
  return createSsrDevServerWithFactories(options, defaultFactories);
}

export async function startSsrDevServerWithFactories(
  options: SsrDevServerOptions,
  factories: SsrDevServerFactories
): Promise<SsrDevServer> {
  const devServer = await createSsrDevServerWithFactories(options, factories);

  try {
    await devServer.listen(options.host);
    return devServer;
  } catch (error) {
    await devServer.close().catch(() => undefined);
    throw error;
  }
}

export async function startSsrDevServer(
  options: SsrDevServerOptions
): Promise<SsrDevServer> {
  return startSsrDevServerWithFactories(options, defaultFactories);
}
