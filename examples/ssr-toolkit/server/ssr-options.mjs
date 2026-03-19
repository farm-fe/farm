import { createSsrRenderConfig } from './template.mjs';

export function createSsrServerOptions(params) {
  const { runtime, hmrPort } = params;
  const configFile = './farm.config.ts';

  return {
    command: runtime.command,
    mode: runtime.mode,
    client: {
      configFile,
      server: {
        preview: {
          distDir: 'dist/client'
        },
        ...(hmrPort
          ? {
              hmr: {
                port: hmrPort,
                host: runtime.host
              }
            }
          : {})
      },
      compilation: {
        minify: false,
        input: {
          index: './index.html'
        },
        output: {
          path: 'dist/client'
        }
      }
    },
    server: {
      configFile,
      compilation: {
        input: {
          index: './src/entry-server.ts'
        },
        output: {
          path: 'dist/server',
          targetEnv: 'node',
          format: 'esm'
        },
        minify: false
      }
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
