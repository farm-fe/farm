import Koa from 'koa';
import koaStatic from 'koa-static';
import path from 'path';

const app = new Koa();

// 拼接静态文件目录的绝对路径
const staticPath = path.join(process.cwd(), './build');

// 使用koa-static中间件处理静态文件
app.use(koaStatic(staticPath));

// 定义路由
app.use(async (ctx) => {
  ctx.body = 'Hello, Koa with koa-static!';
});

const port = process.env.PORT || 3000;

app.listen(port, () => {
  console.log(`Server is running on http://localhost:${port}`);
});
