import solid from 'vite-plugin-solid';
import { generateHydrationScript } from 'solid-js/web';
import path from 'path';
import { pathToFileURL } from 'url';

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
            const curDir =
              process.platform === 'win32'
                ? pathToFileURL(__dirname)
                : __dirname;
            const render = await import(
              path.join(curDir.toString(), 'dist', 'index.js')
            ).then((m) => m['default']);
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
