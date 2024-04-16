import solid from 'vite-plugin-solid';
import { generateHydrationScript } from 'solid-js/web';
import path from 'path';

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
      // modules: {
      //   indentName: 'farm-[name]-[hash]'
      // },
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    },
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
            const render = await import(
              path.join(__dirname, 'dist', 'index.js')
            ).then((m) => m.default);
            const renderedHtml = render(ctx.path);

            const html = template
              .replace('<div>app-html-to-replace</div>', renderedHtml)
              .replace('</head>', generateHydrationScript());
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
