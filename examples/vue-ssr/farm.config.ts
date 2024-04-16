import { defineConfig } from '@farmfe/core';
import vue from '@vitejs/plugin-vue';
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
          const render = await import(
            path.join(__dirname, 'dist', 'index.js')
          ).then((m) => m.default);
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
