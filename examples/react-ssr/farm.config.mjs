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
      (server) => 
        async (req, res, next) => {
          if (!res.writableEnded) {
            // console.log('ctx.path', ctx.path);
            const template = server
              .getCompiler()
              .resource('index_client.html')
              .toString();
            const projectRoot = path.dirname(fileURLToPath(import.meta.url));
            const moudlePath = path.join(projectRoot, 'dist', 'index.js');
            const render = await import(pathToFileURL(moudlePath)).then((m) => m['default']);
            const renderedHtml = render(req.url);
            // console.log(renderedHtml);
            const html = template.replace(
              '{app-html-to-replace}',
              renderedHtml
            );
            res.writeHead(200, { 'Content-Type': 'text/html' }).end(html);
            return;
          }

          console.log('req.url outer', req.url);
          next();
        }
    ]
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
};
