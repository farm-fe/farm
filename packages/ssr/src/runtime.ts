import type { IncomingMessage, ServerResponse } from 'node:http';
import type { SsrPreviewOptions } from './build-preview.js';
import { createSsrPreviewServer } from './build-preview.js';
import {
  resolveSsrConfigForCommand,
  type SsrConfig
} from './config-resolver.js';
import type { SsrDevServerOptions, SsrMiddlewareServer } from './dev-server.js';
import { createSsrDevServer } from './dev-server.js';
import type { SsrRuntimeCommand } from './runtime-types.js';

export type SsrRuntimeConfig = SsrConfig &
  (
    | (SsrDevServerOptions & { command?: 'dev' })
    | (SsrPreviewOptions & { command: 'preview' })
  );

export type SsrRuntime = {
  start(): Promise<void>;
  close(): Promise<void>;
  middlewares: SsrMiddlewareServer;
  render(
    url: string,
    req: IncomingMessage,
    res: ServerResponse
  ): Promise<string>;
};

function resolveCommand(options: SsrRuntimeConfig): SsrRuntimeCommand {
  return options.command ?? 'dev';
}

function resolveMode(
  options: SsrRuntimeConfig,
  command: SsrRuntimeCommand
): string {
  if (options.mode) {
    return options.mode;
  }

  return command === 'preview' ? 'production' : 'development';
}

export async function createSsrRuntime(
  options: SsrRuntimeConfig
): Promise<SsrRuntime> {
  const command = resolveCommand(options);
  const mode = resolveMode(options, command);

  if (command === 'preview') {
    const resolved = resolveSsrConfigForCommand(options, command);
    const previewOptions: SsrPreviewOptions = {
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
    const previewServer = await createSsrPreviewServer(previewOptions);

    return {
      middlewares: previewServer.middlewares,
      render: previewServer.render,
      async start() {
        await previewServer.listen(previewOptions.host);
      },
      async close() {
        await previewServer.close();
      }
    };
  }

  const resolved = resolveSsrConfigForCommand(options, command);
  const devOptions: SsrDevServerOptions = {
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
  const devServer = await createSsrDevServer(devOptions);

  return {
    middlewares: devServer.middlewares,
    render: devServer.render,
    async start() {
      await devServer.listen(devOptions.host);
    },
    async close() {
      await devServer.close();
    }
  };
}
