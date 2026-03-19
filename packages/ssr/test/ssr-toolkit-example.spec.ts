import { describe, expect, it, vi } from 'vitest';
import {
  resolveHostPort,
  resolveOptionalDevHmrPort
} from '../../../examples/ssr-toolkit/server/ports.mjs';
import {
  createRuntimeConfig,
  resolveSsrCommand,
  resolveSsrEnvMode,
  resolveTemplateMode
} from '../../../examples/ssr-toolkit/server/runtime-config.mjs';
import {
  createPingPayload,
  createSsrServerOptions,
  createStartupMessage
} from '../../../examples/ssr-toolkit/server/ssr-options.mjs';
import {
  createSsrRenderConfig,
  createTemplateConfig,
  injectAppHtmlIntoBuiltTemplate,
  renderEjsLike
} from '../../../examples/ssr-toolkit/server/template.mjs';

describe('ssr toolkit example runtime config', () => {
  it('resolves dev defaults from env', () => {
    const config = createRuntimeConfig({});
    expect(config).toMatchObject({
      host: '127.0.0.1',
      command: 'dev',
      mode: 'development',
      templateMode: 'html',
      explicitHmrPort: undefined,
      explicitHostPort: undefined
    });
  });

  it('resolves preview command/mode from env', () => {
    const env = {
      SSR_COMMAND: 'preview'
    };

    expect(resolveSsrCommand(env)).toBe('preview');
    expect(resolveSsrEnvMode('preview', env)).toBe('production');
    expect(resolveTemplateMode(env)).toBe('html');
  });

  it('prefers explicit env mode and ejs template', () => {
    const env = {
      SSR_MODE: 'preview',
      SSR_ENV_MODE: 'staging',
      SSR_TEMPLATE_MODE: 'ejs',
      SSR_HMR_PORT: '9821',
      SSR_HOST_PORT: '3022'
    };

    const config = createRuntimeConfig(env);
    expect(config).toMatchObject({
      command: 'preview',
      mode: 'staging',
      templateMode: 'ejs',
      explicitHmrPort: 9821,
      explicitHostPort: 3022
    });
  });
});

describe('ssr toolkit example port policy', () => {
  it('returns undefined hmr port when command is preview', async () => {
    const hmrPort = await resolveOptionalDevHmrPort({
      command: 'preview',
      host: '127.0.0.1',
      explicitHmrPort: undefined
    });

    expect(hmrPort).toBeUndefined();
  });

  it('rejects invalid explicit hmr port', async () => {
    await expect(
      resolveOptionalDevHmrPort({
        command: 'dev',
        host: '127.0.0.1',
        explicitHmrPort: NaN,
        isPortAvailableFn: vi.fn(async () => true)
      })
    ).rejects.toThrow('invalid SSR_HMR_PORT');
  });

  it('falls back to next available host port', async () => {
    const isPortAvailableFn = vi.fn(async (port: number) => port === 3012);
    const hostPort = await resolveHostPort({
      host: '127.0.0.1',
      explicitHostPort: undefined,
      startPort: 3011,
      maxProbe: 3,
      isPortAvailableFn
    });

    expect(hostPort).toBe(3012);
    expect(isPortAvailableFn).toHaveBeenNthCalledWith(1, 3011, '127.0.0.1');
    expect(isPortAvailableFn).toHaveBeenNthCalledWith(2, 3012, '127.0.0.1');
  });
});

describe('ssr toolkit example template policy', () => {
  it('injects app html into built template root container', () => {
    const html = injectAppHtmlIntoBuiltTemplate(
      '<html><div id=root></div><script src=/index.js></script></html>',
      '<main>app</main>'
    );

    expect(html).toContain('<div id="root"><main>app</main></div>');
    expect(html).toContain('<script src=/index.js></script>');
  });

  it('supports ejs-like transform with escaping and raw html', () => {
    const html = renderEjsLike(
      '<title><%= pageTitle %></title><div><%- appHtml %></div>',
      {
        pageTitle: '<unsafe>',
        appHtml: '<p>ok</p>'
      }
    );

    expect(html).toContain('<title>&lt;unsafe&gt;</title>');
    expect(html).toContain('<div><p>ok</p></div>');
  });

  it('creates preview template config using built index html', async () => {
    const config = createTemplateConfig({
      command: 'preview',
      templateMode: 'html'
    });

    expect(config).toMatchObject({
      file: './dist/client/index.html'
    });
    expect(typeof config.transform).toBe('function');
    await expect(
      config.transform({
        template:
          '<div id=root></div><script src=/index.js data-farm-resource=true></script>',
        appHtml: '<div>preview</div>',
        url: '/',
        req: {} as never,
        res: {} as never,
        root: '/project'
      })
    ).resolves.toContain('data-farm-resource=true');
  });

  it('rejects preview+ejs template mode', () => {
    expect(() =>
      createTemplateConfig({
        command: 'preview',
        templateMode: 'ejs'
      })
    ).toThrow('preview mode does not support SSR_TEMPLATE_MODE=ejs');
  });
});

describe('ssr toolkit example option builders', () => {
  it('creates ssr server options and startup metadata', () => {
    const runtime = createRuntimeConfig({
      SSR_COMMAND: 'preview'
    });

    const options = createSsrServerOptions({
      runtime,
      hmrPort: undefined
    });
    const renderConfig = createSsrRenderConfig({
      command: runtime.command,
      templateMode: runtime.templateMode
    });
    const startupMessage = createStartupMessage({
      runtime,
      hostPort: 3011,
      hmrPort: undefined
    });
    const pingPayload = createPingPayload(runtime);

    expect(options.command).toBe('preview');
    expect(options.client.configFile).toBe('./farm.config.ts');
    expect(options.server.configFile).toBe('./farm.config.ts');
    expect(options.client.server?.preview?.distDir).toBe('dist/client');
    expect(options.client.compilation?.output?.path).toBe('dist/client');
    expect(options.server.compilation?.output?.path).toBe('dist/server');
    expect(options.server.compilation?.output?.targetEnv).toBe('node');
    expect(options.server.compilation?.output?.format).toBe('esm');
    expect(options.ssr.entry).toBeUndefined();
    expect(options.ssr.template.file).toBe('./dist/client/index.html');
    expect(typeof options.ssr.template.transform).toBe('function');
    expect(renderConfig.template.file).toBe('./dist/client/index.html');
    expect(startupMessage).toContain('command=preview');
    expect(pingPayload).toMatchObject({
      command: 'preview',
      mode: 'production',
      templateMode: 'html'
    });
  });

  it('injects hmr port into client config when provided', () => {
    const runtime = createRuntimeConfig({});
    const options = createSsrServerOptions({
      runtime,
      hmrPort: 9821
    });

    expect(options.client.server?.hmr).toMatchObject({
      port: 9821,
      host: runtime.host
    });
  });
});
