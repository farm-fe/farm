import { createSsrRenderConfig } from './template.mjs';

export function createSsrServerOptions(params) {
  const { runtime, hmrPort } = params;

  return {
    command: runtime.command,
    mode: runtime.mode,
    client: {
      configFile: './farm.config.client.ts',
      ...(hmrPort
        ? {
            server: {
              hmr: {
                port: hmrPort,
                host: runtime.host
              }
            }
          }
        : {})
    },
    server: {
      configFile: './farm.config.server.ts'
    },
    ssr: createSsrRenderConfig({
      command: runtime.command,
      templateMode: runtime.templateMode
    })
  };
}

export function createPingPayload(runtime) {
  return {
    ok: true,
    from: 'ssr-toolkit',
    command: runtime.command,
    mode: runtime.mode,
    templateMode: runtime.templateMode
  };
}

export function createStartupMessage(params) {
  const { runtime, hostPort, hmrPort } = params;

  return `ssr toolkit host: http://${runtime.host}:${hostPort} [command=${runtime.command}, mode=${runtime.mode}, template=${runtime.templateMode}${hmrPort ? `, hmr=${hmrPort}` : ''}]`;
}

