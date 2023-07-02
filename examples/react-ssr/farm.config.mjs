/**
 * @type {import('@farmfe/core').UserConfig}
 */
export default {
  compilation: {
    input: {
      index_client: './index.html'
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
    }
    // treeShaking: true,
    // minify: true,
  },
  server: {
    hmr: true,
    cors: true,
    plugins: [
      (server) => {
        server.app().use(async (ctx, next) => {
          await next();

          if (ctx.path === '/' || ctx.status === 404) {
            console.log('ctx.path', ctx.path);
            const template = server
              .getCompiler()
              .resource('index_client.html')
              .toString();
            const render = await import('./dist/index.js').then(
              (m) => m.default
            );
            const renderedHtml = render(ctx.path);
            console.log(renderedHtml);
            const html = template.replace(
              '<div>app-html-to-replace</div>',
              renderedHtml
            );
            ctx.body = html;
            ctx.type = 'text/html';
            ctx.status = 200;
          }

          console.log('ctx.path outer', ctx.path);
        });
      }
    ]
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
};
