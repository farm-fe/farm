import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
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
    middlewares: [
      (server) => {
        server.app().use(async (ctx, next) => {
          await next();
          if (ctx.path === '/' || ctx.status === 404) {
            // console.log('ctx.path', ctx.path);
            const template = server
              .getCompiler()
              .resource('index_client.html')
              .toString();
            const projectRoot = path.dirname(fileURLToPath(import.meta.url));
            const moudlePath = path.join(projectRoot, 'dist', 'index.js');
            const render = await import(pathToFileURL(moudlePath)).then((m) => m['default']);
            const renderedHtml = render(ctx.path);
            // console.log(renderedHtml);
            const html = template.replace(
              '{app-html-to-replace}',
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
