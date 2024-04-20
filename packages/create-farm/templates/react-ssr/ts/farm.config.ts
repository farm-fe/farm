
import path from 'path';
import { defineConfig } from '@farmfe/core';

export default defineConfig({
  compilation: {
    input: {
      index_client: './index.html'
    },
    output: {
      path: './build'
    },
  },
  server: {
    hmr: true,
    cors: true,
    middlewares: [
      // 注册一个中间件，在服务端渲染应用，
      // 然后注入到服务器渲染的标记并返回最终的index.html
      (server) => {
        server.app().use(async (ctx, next) => {
          await next();

          // 处理index.html或单页面应用路由设置
          if (ctx.path === '/' || ctx.status === 404) {
            // 加载服务端入口，并通过ctx.path渲染
            const render = await import(path.join(process.cwd(), 'dist', 'index.js')).then(
              (m) => m.default
            );
            const renderedHtml = render(ctx.path);

            // 通过server.getCompiler()获取编译的index.html内容
            // 这里的html经过编译并注入了所有客户端bundles文件
            const template = server
              .getCompiler()
              .resource('index_client.html')
              .toString();

            // 将占位符替换为渲染好的内容，并将其作为HTML返回
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
  plugins: ['@farmfe/plugin-react']
});
