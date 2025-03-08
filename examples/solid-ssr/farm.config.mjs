import solid from 'vite-plugin-solid';
import { generateHydrationScript } from 'solid-js/web';
import path from 'path';
import { pathToFileURL, fileURLToPath } from 'url';

/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      client: './index.html'
    },
    output: {
      path: './build'
    },
    // sourcemap: true,
    css: {
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
    // minify: false,
    persistentCache: false
    // treeShaking: true,
    // minify: true,babcl
  },
  server: {
    hmr: true,
    cors: true,
    middlewares: [
      (server) => {
        server.app().use(async (ctx, next) => {
          await next();

          if (ctx.path === '/' || ctx.status === 404) {
            console.log('ctx.path', ctx.path);
            const template = server
              .getCompiler()
              .resource('client.html')
              .toString();
            const projectRoot = path.dirname(fileURLToPath(import.meta.url));
            const moudlePath = path.join(projectRoot, 'dist', 'index.js');
            const render = await import(pathToFileURL(moudlePath)).then((m) => m['default']);
            const renderedHtml = render(ctx.path);

            const html = template
              .replace('<div>app-html-to-replace</div>', renderedHtml)
              .replace('<meta hydration />', generateHydrationScript());
            console.log(renderedHtml);
            ctx.body = html;
            ctx.type = 'text/html';
            ctx.status = 200;
          }

          console.log('ctx.path outer', ctx.path);
        });
      }
    ]
  },
  vitePlugins: [
    () => ({
      filters: ['.+'],
      vitePlugin: solid({ solid: { hydratable: true } })
    })
  ]
};
