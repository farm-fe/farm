import path from 'path';
import { fileURLToPath, pathToFileURL } from 'url';
import { renderDev } from '../ssr-utils/render-dev.mjs';
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
          try {
          if (!res.writableEnded) {
            // console.log('ctx.path', ctx.path);
            const template = server
              .getCompiler()
              .resource('index_client.html')
              .toString();
            const projectRoot = path.dirname(fileURLToPath(import.meta.url));
            const moudlePath = path.join(projectRoot, 'dist', 'index.js');
            const renderedHtml = await renderDev(moudlePath, req.url, {
              workerUrl: pathToFileURL(path.join(projectRoot, '../ssr-utils/render-worker.mjs'))
            });
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
          } catch (error) {
            next(error);
          }
        }
    ]
  },
  plugins: ['@farmfe/plugin-react', '@farmfe/plugin-sass']
};
