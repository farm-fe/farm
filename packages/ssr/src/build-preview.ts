import { Buffer } from 'node:buffer';
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

export interface SsrBuildOptions {
  client: FarmCliOptions & UserConfig;
  server: FarmCliOptions & UserConfig;
}

export interface SsrResolvedBuildOutput {
  root: string;
  outputPath: string;
}

export interface SsrPreviewRenderContext {
  url: string;
  req: IncomingMessage;
  res: ServerResponse;
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
}

export interface SsrResolvedPreviewMetadata {
  clientBuildOutput: SsrResolvedBuildOutput;
  serverBuildOutput: SsrResolvedBuildOutput | null;
  templateFilePath: string | null;
  serverEntryFilePath: string | null;
  manifestFileCandidates: string[];
}

export interface SsrPreviewServer {
  middlewares: SsrMiddlewareServer;
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
  createClientPreviewServer(
    config: FarmCliOptions & UserConfig
  ): Promise<SsrPreviewClientServerLike>;
  createHostServer(middlewares: SsrMiddleware): SsrDevHostServerLike;
  readFile(filePath: string): Promise<string>;
  importModule(filePath: string): Promise<Record<string, unknown>>;
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
  createHostServer(middlewares) {
    return createNodeServer(middlewares) as SsrDevHostServerLike;
  }
};

const DEFAULT_PREVIEW_ENTRY = 'index.js';
const PREVIEW_MANIFEST_CANDIDATES = [
  'manifest.json',
  'ssr-manifest.json'
] as const;

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

  const templateFilePath = resolveTemplateFilePath({
    templateFile: template.file,
    clientBuildOutput: params.clientBuildOutput
  });
  return params.factories.readFile(templateFilePath);
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

function createSsrPreviewRenderMiddleware(params: {
  options: SsrPreviewRenderOptions;
  clientBuildOutput: SsrResolvedBuildOutput;
  loadModule: () => Promise<Record<string, unknown>>;
  factories: SsrBuildPreviewFactories;
}) {
  return async (
    req: IncomingMessage,
    res: ServerResponse,
    next: (error?: unknown) => void
  ) => {
    const context: SsrPreviewRenderContext = {
      url: req.url ?? '/',
      req,
      res
    };
    const shouldHandle =
      params.options.shouldHandle ?? ((ctx) => isHtmlRequest(ctx.req));

    if (!shouldHandle(context)) {
      next();
      return;
    }

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
        req,
        res,
        url: context.url,
        factories: params.factories
      });
      const templateOptions = params.options.template ?? {};

      let html: string;
      if (templateOptions.transform) {
        html = await templateOptions.transform({
          url: context.url,
          req,
          res,
          root: params.clientBuildOutput.root,
          appHtml,
          template
        });
      } else {
        const placeholder = templateOptions.placeholder ?? '<!--app-html-->';

        if (!template.includes(placeholder)) {
          throw new Error(
            `[farm ssr] template placeholder "${placeholder}" was not found.`
          );
        }

        html = template.replace(placeholder, appHtml);
      }

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
  await factories.runBuild(options.client);
  await factories.runBuild(options.server);
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
    serverEntryFilePath,
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
