import { Buffer } from 'node:buffer';
import { randomUUID } from 'node:crypto';
import fs from 'node:fs/promises';
import {
  createServer as createNodeServer,
  type IncomingMessage,
  type ServerResponse
} from 'node:http';
import path from 'node:path';
import { pathToFileURL } from 'node:url';
import {
  build,
  type FarmCliOptions,
  PreviewServer,
  resolveConfig,
  type UserConfig
} from '@farmfe/core';
import type {
  SsrDevHostServerLike,
  SsrDevServerListenOptions,
  SsrMiddleware,
  SsrMiddlewareServer,
  SsrNextMiddleware,
  SsrTemplateLoadContext,
  SsrTemplateRenderContext
} from './dev-server.js';
import { toSsrError } from './errors.js';
import {
  injectAssetsIntoHtml,
  resolveAssetsFromManifest,
  type SsrBuildInfo,
  type SsrManifest
} from './manifest.js';
import type { SsrRuntimeHooks } from './runtime-hooks.js';
import type {
  SsrRuntimeAssets,
  SsrRuntimeCommand,
  SsrRuntimeMeta
} from './runtime-types.js';

export interface SsrBuildOptions {
  client: FarmCliOptions & UserConfig;
  server: FarmCliOptions & UserConfig;
  hooks?: SsrRuntimeHooks;
  $client?: FarmCliOptions & UserConfig;
  $server?: FarmCliOptions & UserConfig;
}

export interface SsrResolvedBuildOutput {
  root: string;
  outputPath: string;
}

export interface SsrPreviewRenderContext {
  url: string;
  req: IncomingMessage;
  res: ServerResponse;
  command: SsrRuntimeCommand;
  mode: string;
  runtime: SsrRuntimeMeta;
  assets: SsrRuntimeAssets;
  requestId?: string;
}

export interface SsrPreviewTemplateOptions {
  file?: string;
  placeholder?: string;
  load?: (context: SsrTemplateLoadContext) => string | Promise<string>;
  transform?: (context: SsrTemplateRenderContext) => string | Promise<string>;
}

export interface SsrPreviewRenderOptions {
  entry?: string;
  exportName?: string;
  template?: SsrPreviewTemplateOptions;
  shouldHandle?: (ctx: SsrPreviewRenderContext) => boolean;
  render?: (
    mod: Record<string, unknown>,
    ctx: SsrPreviewRenderContext
  ) => unknown;
  onError?: (
    error: unknown,
    ctx: SsrPreviewRenderContext & { next: (error?: unknown) => void }
  ) => void | Promise<void>;
}

export interface SsrPreviewOptions {
  client: FarmCliOptions & UserConfig;
  server: FarmCliOptions & UserConfig;
  host?: SsrDevServerListenOptions;
  ssrMiddleware?: SsrNextMiddleware;
  ssr?: SsrPreviewRenderOptions;
  hooks?: SsrRuntimeHooks;
  $client?: FarmCliOptions & UserConfig;
  $server?: FarmCliOptions & UserConfig;
}

export interface SsrResolvedPreviewMetadata {
  clientBuildOutput: SsrResolvedBuildOutput;
  serverBuildOutput: SsrResolvedBuildOutput | null;
  templateFilePath: string | null;
  serverEntryFilePath: string | null;
  buildInfoPath: string | null;
  manifestFilePath: string | null;
  manifestFileCandidates: string[];
}

export interface SsrPreviewServer {
  middlewares: SsrMiddlewareServer;
  render(
    url: string,
    req: IncomingMessage,
    res: ServerResponse
  ): Promise<string>;
  listen(options?: SsrDevServerListenOptions): Promise<void>;
  close(): Promise<void>;
}

export interface SsrPreviewClientServerLike {
  middlewares: SsrMiddleware;
  close(): Promise<void>;
}

export interface SsrBuildPreviewFactories {
  runBuild(config: FarmCliOptions & UserConfig): Promise<void>;
  resolveBuildOutput(
    config: FarmCliOptions & UserConfig
  ): Promise<SsrResolvedBuildOutput>;
  resolveClientEntries(
    config: FarmCliOptions & UserConfig
  ): Promise<{ entries: string[]; publicPath: string }>;
  createClientPreviewServer(
    config: FarmCliOptions & UserConfig
  ): Promise<SsrPreviewClientServerLike>;
  createHostServer(middlewares: SsrMiddleware): SsrDevHostServerLike;
  readFile(filePath: string): Promise<string>;
  importModule(filePath: string): Promise<Record<string, unknown>>;
  listFiles(dir: string): Promise<string[]>;
  writeFile(filePath: string, content: string): Promise<void>;
}

function shouldRetryImportWithoutCss(error: unknown) {
  if (!(error instanceof Error)) {
    return false;
  }

  const nodeError = error as Error & { code?: string };
  return (
    nodeError.code === 'ERR_UNKNOWN_FILE_EXTENSION' &&
    error.message.includes('.css')
  );
}

function rewriteModuleImportsForDataUrl(params: {
  code: string;
  filePath: string;
}) {
  const moduleDir = path.dirname(params.filePath);
  const originalFileUrl = pathToFileURL(params.filePath).href;
  const importStatementRE =
    /import\s+(?:[^'"`]*?\s+from\s+)?(['"])([^'"]+)\1\s*;?/g;
  const rewrittenImports = params.code.replace(
    importStatementRE,
    (statement: string, quote: string, specifier: string) => {
      if (specifier.endsWith('.css')) {
        return '';
      }

      if (!specifier.startsWith('.')) {
        return statement;
      }

      const absoluteFileUrl = pathToFileURL(
        path.resolve(moduleDir, specifier)
      ).href;
      return statement.replace(
        `${quote}${specifier}${quote}`,
        `${quote}${absoluteFileUrl}${quote}`
      );
    }
  );

  return rewrittenImports.replaceAll(
    'createRequire(import.meta.url)',
    `createRequire(${JSON.stringify(originalFileUrl)})`
  );
}

export async function importModuleWithCssInterop(
  filePath: string,
  readFile: (filePath: string, encoding: BufferEncoding) => Promise<string>
) {
  const fileUrl = pathToFileURL(filePath).href;

  try {
    return (await import(fileUrl)) as Record<string, unknown>;
  } catch (error) {
    if (!shouldRetryImportWithoutCss(error)) {
      throw error;
    }
  }

  const source = await readFile(filePath, 'utf-8');
  const rewrittenSource = rewriteModuleImportsForDataUrl({
    code: source,
    filePath
  });
  const dataUrl = `data:text/javascript;base64,${Buffer.from(rewrittenSource).toString('base64')}`;

  return (await import(dataUrl)) as Record<string, unknown>;
}

const defaultFactories: SsrBuildPreviewFactories = {
  runBuild(config) {
    return build(config);
  },
  async resolveBuildOutput(config) {
    const resolved = await resolveConfig(config, 'build', 'production');
    const root = resolved.root;
    const rawOutputPath = resolved.compilation.output.path;
    const outputPath = path.isAbsolute(rawOutputPath)
      ? rawOutputPath
      : path.join(root, rawOutputPath);

    return {
      root,
      outputPath
    };
  },
  async resolveClientEntries(config) {
    const resolved = await resolveConfig(config, 'build', 'production');
    const inputs = Object.values(resolved.compilation.input ?? {});
    return {
      entries: inputs.length ? inputs : [],
      publicPath: resolved.compilation.output.publicPath ?? '/'
    };
  },
  async createClientPreviewServer(config) {
    const previewServer = new PreviewServer(config);
    await previewServer.createPreviewServer();

    return {
      middlewares: previewServer.middlewares as unknown as SsrMiddleware,
      close: () => previewServer.close()
    };
  },
  readFile(filePath) {
    return fs.readFile(filePath, 'utf-8');
  },
  importModule(filePath) {
    return importModuleWithCssInterop(filePath, fs.readFile);
  },
  async listFiles(dir) {
    const results: string[] = [];
    const walk = async (current: string) => {
      const entries = await fs.readdir(current, { withFileTypes: true });
      for (const entry of entries) {
        const fullPath = path.join(current, entry.name);
        if (entry.isDirectory()) {
          await walk(fullPath);
          continue;
        }
        if (entry.isFile()) {
          const relPath = path.relative(dir, fullPath);
          results.push(relPath.split(path.sep).join('/'));
        }
      }
    };
    await walk(dir);
    return results;
  },
  writeFile(filePath, content) {
    return fs.writeFile(filePath, content, 'utf-8');
  },
  createHostServer(middlewares) {
    return createNodeServer(middlewares) as SsrDevHostServerLike;
  }
};

const DEFAULT_PREVIEW_ENTRY = 'index.js';
const PREVIEW_MANIFEST_CANDIDATES = [
  'manifest.json',
  'ssr-manifest.json'
] as const;
const SSR_MANIFEST_CLIENT = 'manifest.client.json';
const SSR_MANIFEST_SERVER = 'manifest.server.json';
const SSR_BUILD_INFO = 'build-info.json';

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
  options: SsrPreviewOptions
): Required<SsrDevServerListenOptions> {
  const previewConfig = options.client.server?.preview ?? {};
  const fallbackHost =
    typeof previewConfig.host === 'string' ? previewConfig.host : '127.0.0.1';

  return {
    port: options.host?.port ?? previewConfig.port ?? 1911,
    hostname: options.host?.hostname ?? fallbackHost
  };
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

async function closeWithOrder(params: {
  hostServer: SsrDevHostServerLike;
  clientPreviewServer: SsrPreviewClientServerLike;
}) {
  const results = await Promise.allSettled([
    closeHostServer(params.hostServer),
    params.clientPreviewServer.close()
  ]);
  const rejected = results.find(
    (result): result is PromiseRejectedResult => result.status === 'rejected'
  );

  if (rejected) {
    throw rejected.reason;
  }
}

function resolvePreviewEntryFilePath(params: {
  entry: string;
  serverBuildOutput: SsrResolvedBuildOutput;
}) {
  if (path.isAbsolute(params.entry)) {
    return params.entry;
  }

  return path.join(params.serverBuildOutput.outputPath, params.entry);
}

function resolvePreviewEntry(entry?: string) {
  return entry ?? DEFAULT_PREVIEW_ENTRY;
}

function resolveManifestFileCandidates(
  clientBuildOutput: SsrResolvedBuildOutput
) {
  return PREVIEW_MANIFEST_CANDIDATES.map((filename) =>
    path.join(clientBuildOutput.outputPath, filename)
  );
}

function resolveBuildInfoPath(clientBuildOutput: SsrResolvedBuildOutput) {
  return path.join(clientBuildOutput.outputPath, SSR_BUILD_INFO);
}

function resolveClientManifestPath(clientBuildOutput: SsrResolvedBuildOutput) {
  return path.join(clientBuildOutput.outputPath, SSR_MANIFEST_CLIENT);
}

function resolveServerManifestPath(serverBuildOutput: SsrResolvedBuildOutput) {
  return path.join(serverBuildOutput.outputPath, SSR_MANIFEST_SERVER);
}

function createManifestFromFiles(params: {
  files: string[];
  entries: string[];
}): SsrManifest {
  const jsFiles = params.files.filter(
    (file) =>
      file.endsWith('.js') || file.endsWith('.mjs') || file.endsWith('.cjs')
  );
  const cssFiles = params.files.filter((file) => file.endsWith('.css'));
  const entries: Record<
    string,
    { js: string[]; css: string[]; preload: string[] }
  > = {};

  for (const entry of params.entries) {
    entries[entry] = {
      js: [...jsFiles],
      css: [...cssFiles],
      preload: [...jsFiles]
    };
  }

  if (params.entries.length === 0) {
    entries['__all__'] = {
      js: [...jsFiles],
      css: [...cssFiles],
      preload: [...jsFiles]
    };
  }

  return {
    version: 1,
    entries,
    modules: {}
  };
}

async function writeBuildArtifacts(params: {
  options: SsrBuildOptions;
  factories: SsrBuildPreviewFactories;
}): Promise<void> {
  const clientBuildOutput = await params.factories.resolveBuildOutput(
    params.options.client
  );
  const serverBuildOutput = await params.factories.resolveBuildOutput(
    params.options.server
  );
  const { entries } = await params.factories.resolveClientEntries(
    params.options.client
  );
  const outputFiles = await params.factories.listFiles(
    clientBuildOutput.outputPath
  );
  const manifest = createManifestFromFiles({
    files: outputFiles,
    entries
  });
  const manifestPath = resolveClientManifestPath(clientBuildOutput);
  const serverManifestPath = resolveServerManifestPath(serverBuildOutput);
  const serverEntry = path.join(
    serverBuildOutput.outputPath,
    DEFAULT_PREVIEW_ENTRY
  );
  const buildInfo: SsrBuildInfo = {
    version: 1,
    client: {
      outputDir: clientBuildOutput.outputPath,
      manifest: manifestPath,
      entry: entries[0] ?? undefined
    },
    server: {
      outputDir: serverBuildOutput.outputPath,
      entry: serverEntry
    }
  };

  await params.factories.writeFile(
    manifestPath,
    JSON.stringify(manifest, null, 2)
  );
  await params.factories.writeFile(
    serverManifestPath,
    JSON.stringify({ version: 1, entries: {}, modules: {} }, null, 2)
  );
  await params.factories.writeFile(
    resolveBuildInfoPath(clientBuildOutput),
    JSON.stringify(buildInfo, null, 2)
  );
}

function resolveTemplateFilePath(params: {
  templateFile?: string;
  clientBuildOutput: SsrResolvedBuildOutput;
}) {
  if (!params.templateFile) {
    return path.join(params.clientBuildOutput.outputPath, 'index.html');
  }

  if (path.isAbsolute(params.templateFile)) {
    return params.templateFile;
  }

  return path.join(params.clientBuildOutput.root, params.templateFile);
}

function resolveTemplateFileCandidates(params: {
  templateFile?: string;
  clientBuildOutput: SsrResolvedBuildOutput;
}) {
  const defaultFilePath = path.join(
    params.clientBuildOutput.outputPath,
    'index.html'
  );

  if (!params.templateFile) {
    return [defaultFilePath];
  }

  if (path.isAbsolute(params.templateFile)) {
    return [params.templateFile];
  }

  const normalizedTemplateFile = params.templateFile.replace(/^\/+/, '');
  const candidates = [
    path.join(params.clientBuildOutput.outputPath, normalizedTemplateFile),
    path.join(params.clientBuildOutput.root, params.templateFile)
  ];

  return [...new Set(candidates)];
}

async function loadTemplateContent(params: {
  options: SsrPreviewRenderOptions;
  clientBuildOutput: SsrResolvedBuildOutput;
  req: IncomingMessage;
  res: ServerResponse;
  url: string;
  factories: SsrBuildPreviewFactories;
}) {
  const template = params.options.template ?? {};
  const loadContext: SsrTemplateLoadContext = {
    url: params.url,
    req: params.req,
    res: params.res,
    root: params.clientBuildOutput.root
  };

  if (template.load) {
    return await template.load(loadContext);
  }

  const templateFileCandidates = resolveTemplateFileCandidates({
    templateFile: template.file,
    clientBuildOutput: params.clientBuildOutput
  });
  let lastReadError: unknown;

  for (const filePath of templateFileCandidates) {
    try {
      return await params.factories.readFile(filePath);
    } catch (error) {
      lastReadError = error;
      const code = (error as { code?: unknown })?.code;
      if (code !== 'ENOENT') {
        throw error;
      }
    }
  }

  throw lastReadError;
}

function resolveRenderResult(params: {
  options: SsrPreviewRenderOptions;
  moduleExports: Record<string, unknown>;
  context: SsrPreviewRenderContext;
}) {
  if (params.options.render) {
    return params.options.render(params.moduleExports, params.context);
  }

  const exportName = params.options.exportName ?? 'default';
  const render = params.moduleExports[exportName];
  const entry = resolvePreviewEntry(params.options.entry);

  if (typeof render !== 'function') {
    throw new Error(
      `[farm ssr] export "${exportName}" from "${entry}" must be a function or provide ssr.render().`
    );
  }

  return (render as (url: string, ctx: unknown) => unknown)(
    params.context.url,
    params.context
  );
}

export async function renderSsrPreviewHtml(params: {
  options: SsrPreviewRenderOptions;
  clientBuildOutput: SsrResolvedBuildOutput;
  loadModule: () => Promise<Record<string, unknown>>;
  factories: SsrBuildPreviewFactories;
  url: string;
  req: IncomingMessage;
  res: ServerResponse;
  mode: string;
  runtime: SsrRuntimeMeta;
  assets?: SsrRuntimeAssets;
  hooks?: SsrRuntimeHooks;
  requestId?: string;
}): Promise<string> {
  const requestId = params.requestId ?? randomUUID();
  const context: SsrPreviewRenderContext = {
    url: params.url,
    req: params.req,
    res: params.res,
    command: 'preview',
    mode: params.mode,
    runtime: params.runtime,
    assets: params.assets ?? { css: [], preload: [], scripts: [] },
    requestId
  };

  const startTime = Date.now();
  params.hooks?.onRenderStart?.({
    requestId,
    url: context.url,
    command: context.command,
    mode: context.mode,
    runtime: context.runtime
  });

  try {
    const moduleExports = await params.loadModule();
    const renderResult = await resolveRenderResult({
      options: params.options,
      moduleExports,
      context
    });
    const appHtml =
      typeof renderResult === 'string' ? renderResult : String(renderResult);
    const template = await loadTemplateContent({
      options: params.options,
      clientBuildOutput: params.clientBuildOutput,
      req: params.req,
      res: params.res,
      url: context.url,
      factories: params.factories
    });
    const templateOptions = params.options.template ?? {};

    if (templateOptions.transform) {
      const html = await templateOptions.transform({
        url: context.url,
        req: params.req,
        res: params.res,
        root: params.clientBuildOutput.root,
        appHtml,
        template
      });
      params.hooks?.onRenderEnd?.({
        requestId,
        url: context.url,
        command: context.command,
        mode: context.mode,
        runtime: context.runtime,
        ms: Date.now() - startTime
      });
      return html;
    }

    const placeholder = templateOptions.placeholder ?? '<!--app-html-->';

    if (!template.includes(placeholder)) {
      throw new Error(
        `[farm ssr] template placeholder "${placeholder}" was not found.`
      );
    }

    const html = template.replace(placeholder, appHtml);
    const injected = injectAssetsIntoHtml({ html, assets: context.assets });
    params.hooks?.onAssetInject?.({
      requestId,
      url: context.url,
      css: context.assets.css.length,
      preload: context.assets.preload.length,
      scripts: context.assets.scripts.length
    });
    params.hooks?.onRenderEnd?.({
      requestId,
      url: context.url,
      command: context.command,
      mode: context.mode,
      runtime: context.runtime,
      ms: Date.now() - startTime
    });
    return injected;
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
    throw error;
  }
}

async function loadSsrManifest(params: {
  metadata: SsrResolvedPreviewMetadata;
  factories: SsrBuildPreviewFactories;
}): Promise<SsrManifest | null> {
  const candidates: string[] = [];
  if (params.metadata.manifestFilePath) {
    candidates.push(params.metadata.manifestFilePath);
  }
  candidates.push(...params.metadata.manifestFileCandidates);

  for (const filePath of candidates) {
    try {
      const content = await params.factories.readFile(filePath);
      return JSON.parse(content) as SsrManifest;
    } catch (error) {
      const code = (error as { code?: unknown })?.code;
      if (code !== 'ENOENT') {
        throw error;
      }
    }
  }

  return null;
}

function resolveManifestEntryKey(params: {
  buildInfo: SsrBuildInfo | null;
  manifest: SsrManifest | null;
  renderEntry?: string;
}): string | null {
  if (params.buildInfo?.client?.entry) {
    return params.buildInfo.client.entry;
  }

  if (params.renderEntry) {
    return params.renderEntry;
  }

  if (params.manifest) {
    const keys = Object.keys(params.manifest.entries);
    if (keys.length) {
      return keys[0];
    }
  }

  return null;
}

async function loadBuildInfo(params: {
  metadata: SsrResolvedPreviewMetadata;
  factories: SsrBuildPreviewFactories;
}): Promise<SsrBuildInfo | null> {
  if (!params.metadata.buildInfoPath) {
    return null;
  }

  try {
    const content = await params.factories.readFile(
      params.metadata.buildInfoPath
    );
    return JSON.parse(content) as SsrBuildInfo;
  } catch (error) {
    const code = (error as { code?: unknown })?.code;
    if (code !== 'ENOENT') {
      throw error;
    }
  }

  return null;
}

function createSsrPreviewRenderMiddleware(params: {
  options: SsrPreviewRenderOptions;
  clientBuildOutput: SsrResolvedBuildOutput;
  loadModule: () => Promise<Record<string, unknown>>;
  factories: SsrBuildPreviewFactories;
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
    const context: SsrPreviewRenderContext = {
      url: req.url ?? '/',
      req,
      res,
      command: 'preview',
      mode: params.mode,
      runtime: params.runtime,
      assets: { css: [], preload: [], scripts: [] }
    };
    const shouldHandle =
      params.options.shouldHandle ?? ((ctx) => isHtmlRequest(ctx.req));

    if (!shouldHandle(context)) {
      next();
      return;
    }

    try {
      const html = await renderSsrPreviewHtml({
        options: params.options,
        clientBuildOutput: params.clientBuildOutput,
        loadModule: params.loadModule,
        factories: params.factories,
        url: context.url,
        req,
        res,
        mode: params.mode,
        runtime: params.runtime,
        assets: params.assets,
        hooks: params.hooks
      });

      res.statusCode = 200;
      res.setHeader('Content-Type', 'text/html; charset=utf-8');
      res.end(html);
    } catch (error) {
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

export async function buildSsrAppWithFactories(
  options: SsrBuildOptions,
  factories: SsrBuildPreviewFactories
) {
  const resolvedClient = options.$client
    ? { ...options.client, ...options.$client }
    : options.client;
  const resolvedServer = options.$server
    ? { ...options.server, ...options.$server }
    : options.server;
  const clientStart = Date.now();
  options.hooks?.onCompileStart?.({ kind: 'client' });
  await factories.runBuild(resolvedClient);
  options.hooks?.onCompileEnd?.({
    kind: 'client',
    ms: Date.now() - clientStart
  });

  const serverStart = Date.now();
  options.hooks?.onCompileStart?.({ kind: 'server' });
  await factories.runBuild(resolvedServer);
  options.hooks?.onCompileEnd?.({
    kind: 'server',
    ms: Date.now() - serverStart
  });
  await writeBuildArtifacts({
    options: {
      ...options,
      client: resolvedClient,
      server: resolvedServer
    },
    factories
  });
}

export async function buildSsrApp(options: SsrBuildOptions) {
  await buildSsrAppWithFactories(options, defaultFactories);
}

export async function createSsrPreviewServerWithFactories(
  options: SsrPreviewOptions,
  factories: SsrBuildPreviewFactories
): Promise<SsrPreviewServer> {
  if (options.ssr && options.ssrMiddleware) {
    throw new Error(
      '[farm ssr] "ssr" and "ssrMiddleware" cannot be used together.'
    );
  }

  const previewMetadata = await resolveSsrPreviewMetadataWithFactories(
    options,
    factories
  );
  const clientBuildOutput = previewMetadata.clientBuildOutput;
  const serverBuildOutput = previewMetadata.serverBuildOutput;
  const buildInfo = await loadBuildInfo({
    metadata: previewMetadata,
    factories
  });
  const manifest = await loadSsrManifest({
    metadata: previewMetadata,
    factories
  });
  const { publicPath } = await factories.resolveClientEntries(options.client);

  let clientPreviewServer: SsrPreviewClientServerLike | null = null;

  try {
    clientPreviewServer = await factories.createClientPreviewServer(
      options.client
    );
  } catch (error) {
    await clientPreviewServer?.close().catch(() => undefined);
    throw error;
  }

  const middlewares = createMiddlewareServer();
  const runtimeMeta: SsrRuntimeMeta = {
    root: clientBuildOutput.root,
    publicPath
  };
  const runtimeMode = options.client.mode ?? 'production';
  const manifestEntryKey = resolveManifestEntryKey({
    buildInfo,
    manifest,
    renderEntry: options.ssr?.entry
  });
  const resolvedAssets = resolveAssetsFromManifest({
    manifest,
    entry: manifestEntryKey,
    usedModuleIds: [],
    publicPath: runtimeMeta.publicPath
  });

  if (options.ssrMiddleware) {
    middlewares.use(options.ssrMiddleware);
  } else if (
    options.ssr &&
    previewMetadata.serverEntryFilePath &&
    serverBuildOutput
  ) {
    const renderOptions = {
      ...options.ssr,
      entry: resolvePreviewEntry(options.ssr.entry)
    };
    const entryFile = previewMetadata.serverEntryFilePath;
    let loadedModulePromise: Promise<Record<string, unknown>> | null = null;

    middlewares.use(
      createSsrPreviewRenderMiddleware({
        options: renderOptions,
        clientBuildOutput,
        factories,
        runtime: runtimeMeta,
        mode: runtimeMode,
        assets: resolvedAssets,
        hooks: options.hooks,
        loadModule() {
          if (!loadedModulePromise) {
            loadedModulePromise = factories.importModule(entryFile);
          }

          return loadedModulePromise;
        }
      })
    );
  }

  middlewares.use(clientPreviewServer.middlewares);

  const hostServer = factories.createHostServer(middlewares);
  const defaultListenOptions = resolveDefaultListenOptions(options);
  let closed = false;

  return {
    middlewares,
    async render(url, req, res) {
      if (
        !options.ssr ||
        !previewMetadata.serverEntryFilePath ||
        !serverBuildOutput
      ) {
        throw new Error(
          '[farm ssr] ssr options are required to call render() in preview.'
        );
      }
      const renderOptions = {
        ...options.ssr,
        entry: resolvePreviewEntry(options.ssr.entry)
      };
      const entryFile = previewMetadata.serverEntryFilePath;
      let loadedModulePromise: Promise<Record<string, unknown>> | null = null;
      return renderSsrPreviewHtml({
        options: renderOptions,
        clientBuildOutput,
        factories,
        url,
        req,
        res,
        mode: runtimeMode,
        runtime: runtimeMeta,
        assets: resolvedAssets,
        hooks: options.hooks,
        loadModule() {
          if (!loadedModulePromise) {
            loadedModulePromise = factories.importModule(entryFile);
          }
          return loadedModulePromise;
        }
      });
    },
    async listen(listenOptions) {
      if (closed) {
        throw new Error('[farm ssr] preview server is already closed.');
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
        clientPreviewServer
      });
    }
  };
}

export async function createSsrPreviewServer(
  options: SsrPreviewOptions
): Promise<SsrPreviewServer> {
  return createSsrPreviewServerWithFactories(options, defaultFactories);
}

export async function resolveSsrPreviewMetadataWithFactories(
  options: SsrPreviewOptions,
  factories: SsrBuildPreviewFactories
): Promise<SsrResolvedPreviewMetadata> {
  const clientBuildOutput = await factories.resolveBuildOutput(options.client);
  const serverBuildOutput = options.ssr
    ? await factories.resolveBuildOutput(options.server)
    : null;
  const buildInfoPath = resolveBuildInfoPath(clientBuildOutput);
  let buildInfo: SsrBuildInfo | null = null;
  let manifestFilePath: string | null = null;

  try {
    const buildInfoContent = await factories.readFile(buildInfoPath);
    buildInfo = JSON.parse(buildInfoContent) as SsrBuildInfo;
    if (buildInfo?.client?.manifest) {
      manifestFilePath = buildInfo.client.manifest;
    }
  } catch (error) {
    const code = (error as { code?: unknown })?.code;
    if (code !== 'ENOENT') {
      throw error;
    }
  }
  const templateFilePath = options.ssr
    ? resolveTemplateFilePath({
        templateFile: options.ssr.template?.file,
        clientBuildOutput
      })
    : null;
  const serverEntryFilePath =
    options.ssr && serverBuildOutput
      ? resolvePreviewEntryFilePath({
          entry: resolvePreviewEntry(options.ssr.entry),
          serverBuildOutput
        })
      : null;

  return {
    clientBuildOutput,
    serverBuildOutput,
    templateFilePath,
    serverEntryFilePath: buildInfo?.server?.entry ?? serverEntryFilePath,
    buildInfoPath: buildInfo ? buildInfoPath : null,
    manifestFilePath,
    manifestFileCandidates: resolveManifestFileCandidates(clientBuildOutput)
  };
}

export async function resolveSsrPreviewMetadata(
  options: SsrPreviewOptions
): Promise<SsrResolvedPreviewMetadata> {
  return resolveSsrPreviewMetadataWithFactories(options, defaultFactories);
}

export async function startSsrPreviewServerWithFactories(
  options: SsrPreviewOptions,
  factories: SsrBuildPreviewFactories
): Promise<SsrPreviewServer> {
  const previewServer = await createSsrPreviewServerWithFactories(
    options,
    factories
  );

  try {
    await previewServer.listen(options.host);
    return previewServer;
  } catch (error) {
    await previewServer.close().catch(() => undefined);
    throw error;
  }
}

export async function startSsrPreviewServer(
  options: SsrPreviewOptions
): Promise<SsrPreviewServer> {
  return startSsrPreviewServerWithFactories(options, defaultFactories);
}

export async function previewSsrApp(options: SsrPreviewOptions) {
  return startSsrPreviewServer(options);
}
