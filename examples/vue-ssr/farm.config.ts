import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
import { pathToFileURL } from 'node:url';
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
      (server) => async (ctx, next) => {
        await next();
        if (ctx.path === '/' || ctx.status === 404) {
          console.log('ctx.path', ctx.path);
          const template = server
            .getCompiler()
            .resource('client.html')
            .toString();
          const curDir = process.platform === 'win32' ? pathToFileURL(__dirname) : __dirname;
          const render = await import(
            path.join(curDir.toString(), 'dist', 'index.js')
          ).then((m) => m['default']);
          const renderedHtml = await render(ctx.path);
          const html = template.replace(
            '<div>app-html-to-replace</div>',
            renderedHtml
          );
          ctx.body = html;
          ctx.type = 'text/html';
          ctx.status = 200;
        }
        console.log('ctx.path outer', ctx.path);
      }
    ]
  },
  vitePlugins: [vue()]
});
