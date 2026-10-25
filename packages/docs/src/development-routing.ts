import type { DevServerMiddleware } from '@farmfe/core';

const pathname = (value: string) => {
  const origin = 'http://farm-docs.invalid';
  const route = new URL(value.startsWith('/') ? origin + value : value, origin)
    .pathname;
  // Match the Rust-generated client aliases: bare path or one trailing slash.
  return route.length > 1 && route.endsWith('/') && !route.endsWith('//')
    ? route.slice(0, -1)
    : route;
};

/** HTTP transport consumes the Rust-published generation; it never scans content. */
export const developmentRouteMiddleware: DevServerMiddleware =
  (server) => async (request, response, next) => {
    const accept = request.headers.accept;
    if (
      !['GET', 'HEAD'].includes(request.method ?? '') ||
      (accept && !accept.includes('text/html') && !accept.includes('*/*'))
    )
      return next();
    try {
      await server.waitForRuntimeReady();
      const runtime = server.runtime;
      const bytes = runtime?.resource('docs-manifest.json');
      if (!bytes)
        throw new Error('[DOCS_DEV_ROUTES] published manifest missing');
      const manifest = JSON.parse(bytes.toString()) as {
        routePaths?: unknown;
        template?: string | null;
      };
      if (
        !Array.isArray(manifest.routePaths) ||
        !manifest.routePaths.every(
          (route) =>
            typeof route === 'string' &&
            route.startsWith('/') &&
            !route.startsWith('//')
        )
      )
        throw new Error('[DOCS_DEV_ROUTES] published route table invalid');
      // Core has already stripped basePath from request.url. Connect retains
      // the original URL, which matches Rust's public locale/version paths.
      const current = pathname(request.originalUrl ?? request.url ?? '/');
      if (
        manifest.routePaths.some((route: string) => pathname(route) === current)
      )
        return next();
      const html = runtime?.resource(manifest.template ?? 'index.html');
      if (!html)
        throw new Error('[DOCS_DEV_ROUTES] published HTML template missing');
      response.writeHead(404, {
        ...server.config.server?.headers,
        'Content-Type': 'text/html; charset=utf-8',
        'Content-Length': html.length,
        'Cache-Control': 'no-cache'
      });
      response.end(request.method === 'HEAD' ? undefined : html);
    } catch (error) {
      next(error);
    }
  };
