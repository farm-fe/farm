import fs from 'node:fs/promises';
import { createRequire } from 'node:module';
import path from 'node:path';
import type {
  FarmCliOptions,
  ResolvedUserConfig,
  UserConfig
} from '@farmfe/core';
import { RuntimeServerOwner, resolveConfig, Server } from '@farmfe/core';
import docs from '@farmfe/plugin-docs';
import seo from '@farmfe/plugin-docs-seo';
import react from '@farmfe/plugin-react';
import type { SiteConfig } from './config.generated.js';
import { registerDevelopmentLifetime } from './development-lifetime.js';
import { developmentRouteMiddleware } from './development-routing.js';
import type { DocsFarmConfig } from './index.js';
import { assertNativePlugins } from './index.js';

export interface DocsDevelopmentHandle {
  url: string;
  close(): Promise<void>;
}

async function resolveDevelopment(
  site: SiteConfig,
  farm: DocsFarmConfig & FarmCliOptions,
  source?: ResolvedUserConfig
) {
  assertNativePlugins(farm);
  const { snippets, ...coreFarm } = farm;
  const { default: snippetsPlugin } = await import(
    '@farmfe/plugin-docs-snippets'
  );
  const root = await fs.realpath(farm.root ?? process.cwd());
  const consumer = createRequire(path.join(root, 'package.json'));
  const framework = createRequire(import.meta.url);
  const alias: Record<string, string> = {};
  for (const name of ['react', 'react-dom'])
    alias[name] = path.dirname(consumer.resolve(`${name}/package.json`));
  alias['react-refresh$'] = framework.resolve('react-refresh');
  for (const name of [
    '@farmfe/docs/dev-client',
    '@farmfe/docs/client',
    '@farmfe/docs/theme',
    '@farmfe/docs/mdx',
    '@farmfe/docs/snippets',
    '@farmfe/docs/snippets-runtime',
    '@farmfe/docs-theme-default',
    '@farmfe/docs-theme-default/style.css',
    '@farmfe/plugin-syntax-highlight/style.css'
  ])
    alias[`${name}$`] = framework.resolve(name);
  const inline = {
    ...coreFarm,
    root,
    configFile: false as const,
    server: {
      ...farm.server,
      middlewares: [
        ...(farm.server?.middlewares ?? []),
        developmentRouteMiddleware
      ]
    },
    compilation: {
      ...farm.compilation,
      input: { index: 'docs:dev' },
      output: {
        ...farm.compilation?.output,
        publicPath: site.basePath ?? '/'
      },
      lazyCompilation: false,
      resolve: {
        ...farm.compilation?.resolve,
        alias: Array.isArray(farm.compilation?.resolve?.alias)
          ? [
              ...Object.entries(alias).map(([find, replacement]) => ({
                find: find.endsWith('$')
                  ? new RegExp(
                      `^${find.slice(0, -1).replace(/[.*+?^${}()|[\]\\]/g, '\\$&')}$`
                    )
                  : find,
                replacement
              })),
              ...farm.compilation.resolve.alias
            ]
          : { ...farm.compilation?.resolve?.alias, ...alias }
      }
    },
    plugins: [
      docs({ site, development: true }),
      snippetsPlugin(snippets),
      seo({ development: true }),
      react({ development: true, refresh: true }),
      ...(farm.plugins ?? [])
    ]
  };
  const resolved = await resolveConfig(inline, 'dev', 'development');
  if (source) {
    resolved.configFilePath = source.configFilePath;
    resolved.configFileDependencies = source.configFileDependencies;
    resolved.envFiles = source.envFiles;
  }
  return { inline, resolved };
}

/** Transport and lifecycle only; native plugins own content and route semantics. */
export async function startDocsDevelopment(
  site: SiteConfig,
  farm: DocsFarmConfig & FarmCliOptions = {},
  execution: { signal?: AbortSignal; sourceConfig?: ResolvedUserConfig } = {}
): Promise<DocsDevelopmentHandle> {
  const { signal, sourceConfig } = execution;
  const checkAbort = () => {
    if (signal?.aborted)
      throw new Error('[DOCS_CANCELLED] development cancelled', {
        cause: signal.reason
      });
  };
  checkAbort();
  const owner = new RuntimeServerOwner<Server>();
  let closing: Promise<void> | undefined;
  let unregisterLifetime = () => {};
  let lastOrigin = '';
  const rememberActiveOrigin = () => {
    const current = owner.current;
    if (current?.httpServer?.listening) lastOrigin = origin(current);
  };
  const close = () => {
    rememberActiveOrigin();
    signal?.removeEventListener('abort', abort);
    return (closing ??= owner.close().finally(() => unregisterLifetime()));
  };
  const abort = () => {
    void close().catch((error) => console.error('[DOCS_DEV_CLOSE]', error));
  };
  if (!farm.server?.middlewareMode) {
    unregisterLifetime = registerDevelopmentLifetime(close);
  }
  signal?.addEventListener('abort', abort, { once: true });
  const reload = sourceConfig?.configFilePath
    ? async () => {
        const original = sourceConfig as ResolvedUserConfig & {
          transformInlineConfig?: UserConfig & FarmCliOptions;
        };
        const next = await resolveConfig(
          {
            ...original.transformInlineConfig,
            root: sourceConfig.root,
            configFile: sourceConfig.configFilePath
          },
          'dev',
          'development'
        );
        if (
          next.workflow?.module !== '@farmfe/docs/workflow' ||
          !next.workflow.options?.site
        )
          throw new Error(
            '[DOCS_WORKFLOW] development reload requires a Docs configuration'
          );
        const nextFarm = (next.workflow.options.farm ?? {}) as DocsFarmConfig;
        return await resolveDevelopment(
          next.workflow.options.site as SiteConfig,
          {
            ...nextFarm,
            root: next.root,
            customLogger: next.customLogger,
            server: { ...nextFarm.server, ...next.server }
          },
          next
        );
      }
    : undefined;
  const create = async (
    configured: Awaited<ReturnType<typeof resolveDevelopment>>,
    lifetime: AbortSignal,
    prepareOnly = false
  ): Promise<Server> => {
    let server: Server | undefined;
    const check = () => {
      checkAbort();
      if (lifetime.aborted)
        throw new Error('[DOCS_CANCELLED] development cancelled');
    };
    const cancel = () => {
      if (server)
        void server
          .close()
          .catch((error) => console.error('[DOCS_DEV_CLOSE]', error));
    };
    lifetime.addEventListener('abort', cancel, { once: true });
    try {
      check();
      server = await Server.createServer(
        configured.inline,
        configured.resolved,
        {
          runtime: true,
          handleSignals: false,
          deferWebSocketListen: prepareOnly,
          onConfigChange: reload
            ? async () => {
                await owner.replace(async (nextLifetime) => {
                  const candidate = await reload();
                  const active = owner.current;
                  if (!active)
                    throw new Error('[DOCS_DEV_CLOSED] no active service');
                  rememberActiveOrigin();
                  if (active.canReloadRuntimeConfig(candidate.resolved)) {
                    await active.reloadRuntimeConfig(candidate.resolved);
                    return active;
                  }
                  const samePort =
                    candidate.resolved.server?.port ===
                    active.serverOptions.port;
                  const replacement = await create(
                    candidate,
                    nextLifetime,
                    true
                  );
                  const cancelReplacement = () => {
                    void replacement
                      .close()
                      .catch((error) =>
                        console.error('[DOCS_DEV_CLOSE]', error)
                      );
                  };
                  nextLifetime.addEventListener('abort', cancelReplacement, {
                    once: true
                  });
                  let suspended = false;
                  let hmrSuspended = false;
                  try {
                    if (nextLifetime.aborted)
                      throw new Error('[DOCS_CANCELLED] development cancelled');
                    if (samePort) {
                      await active.suspendRuntimeListener();
                      suspended = true;
                      if (nextLifetime.aborted)
                        throw new Error(
                          '[DOCS_CANCELLED] development cancelled'
                        );
                    }
                    hmrSuspended = active.ws.suspendListener();
                    await replacement.listen();
                    if (nextLifetime.aborted)
                      throw new Error('[DOCS_CANCELLED] development cancelled');
                    await active.ws.sendAndFlush({
                      type: 'full-reload',
                      origin: origin(replacement)
                    });
                    return replacement;
                  } catch (error) {
                    const errors: unknown[] = [error];
                    try {
                      await replacement.close();
                    } catch (cleanupError) {
                      errors.push(cleanupError);
                    }
                    if (suspended && !nextLifetime.aborted) {
                      try {
                        await active.resumeRuntimeListener();
                      } catch (restoreError) {
                        errors.push(restoreError);
                      }
                    }
                    if (hmrSuspended && !nextLifetime.aborted) {
                      try {
                        await active.ws.listen();
                      } catch (restoreError) {
                        errors.push(restoreError);
                      }
                    }
                    if (errors.length > 1)
                      throw new AggregateError(
                        errors,
                        '[DOCS_DEV_HANDOFF] candidate failed; cleanup or listener restoration also failed'
                      );
                    throw error;
                  } finally {
                    nextLifetime.removeEventListener(
                      'abort',
                      cancelReplacement
                    );
                  }
                });
                if (owner.current) lastOrigin = origin(owner.current);
                owner.current?.printUrls();
              }
            : undefined
        }
      );
      // The Rust-produced manifest is the identity of the published resources.
      // A page can miss an update while navigating before its new socket opens.
      const synchronizedServer = server;
      server.ws.on('farm-docs:sync', (_data, client) => {
        const bytes =
          synchronizedServer.runtime?.resource('docs-manifest.json');
        if (!bytes) return;
        try {
          const manifest = JSON.parse(bytes.toString()) as {
            snapshotId?: unknown;
          };
          if (typeof manifest.snapshotId === 'string' && manifest.snapshotId)
            client.send('farm-docs:sync', { snapshotId: manifest.snapshotId });
        } catch (error) {
          console.error('[DOCS_DEVELOPMENT_SYNC]', error);
        }
      });
      check();
      if (prepareOnly) await server.prepareRuntime();
      else await server.listen();
      check();
      return server;
    } catch (error) {
      await server?.close();
      throw error;
    } finally {
      lifetime.removeEventListener('abort', cancel);
    }
  };
  const origin = (server: Server): string => {
    const address = server.httpServer?.address();
    if (!address || typeof address === 'string')
      throw new Error('[DOCS_DEV_LISTEN] expected a TCP address');
    const host = ['::', '0.0.0.0'].includes(address.address)
      ? 'localhost'
      : address.address.includes(':')
        ? `[${address.address}]`
        : address.address;
    return `${server.httpsOptions ? 'https' : 'http'}://${host}:${address.port}/`;
  };
  try {
    const server = await owner.replace(async (lifetime) => {
      checkAbort();
      return create(
        await resolveDevelopment(site, farm, sourceConfig),
        lifetime
      );
    });
    lastOrigin = origin(server);
    server.printUrls();
    return {
      get url() {
        rememberActiveOrigin();
        return lastOrigin;
      },
      close
    };
  } catch (error) {
    await close();
    throw error;
  }
}
