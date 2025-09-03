import { defineConfig } from 'farm';
import vue from '@vitejs/plugin-vue';
import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'path';

export default defineConfig({
  compilation: {
    input: {
      client: './index.html'
    },
    output: {
      targetEnv: 'browser',
      path: './build'
    },
    persistentCache: false,
    css: {
      prefixer: {
        targets: ['last 2 versions', 'Firefox ESR', '> 1%', 'ie >= 11']
      }
    }
  },
  server: {
    hmr: true,
    cors: true,
    middlewares: [
      (server) => async (req, res, next) => {
        if (!res.writableEnded) {
          console.log('ctx.url', req.url);
          const template = server
            .getCompiler()
            .resource('client.html')
            .toString();
          const projectRoot = path.dirname(fileURLToPath(import.meta.url));
          const moudlePath = path.join(projectRoot, 'dist', 'index.js');
          const render = await import(pathToFileURL(moudlePath).toString()).then((m) => m['default']);
          const renderedHtml = await render(req.url);
          const html = template.replace(
            '<div>app-html-to-replace</div>',
            renderedHtml
          );
          console.log('write', req.url, res.writableEnded, res.statusCode, res.statusMessage);
          res.writeHead(200, {
            'Content-Type': 'text/html'
          }).end(html);
          return;
        }

        console.log('ctx.path outer', req.url);

        next();
      }
    ]
  },
  vitePlugins: [vue()]
});
