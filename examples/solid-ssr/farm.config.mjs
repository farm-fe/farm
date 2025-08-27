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
      (server) => async (req, res, next) => {
        console.log('req.url pre', req.url, res.statusCode);

        if (!res.writableEnded) {
          console.log('req.url', req.url);
          const template = server
            .getCompiler()
            .resource('client.html')
            .toString();
          const projectRoot = path.dirname(fileURLToPath(import.meta.url));
          const moudlePath = path.join(projectRoot, 'dist', 'index.js');
          const render = await import(pathToFileURL(moudlePath)).then((m) => m['default']);
          const renderedHtml = render(req.url);

          const html = template
            .replace('<div>app-html-to-replace</div>', renderedHtml)
            .replace('{hydrationScript}', generateHydrationScript());

          res.writeHead(200, {
            'Content-Type': 'text/html'
          }).end(html);
          return;
        }

        console.log('req.url outer', req.url, res.statusCode);
        next();
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
