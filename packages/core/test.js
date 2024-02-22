import Koa from 'koa';
import serve from 'koa-static';
import compress from 'koa-compress';
import mount from 'koa-mount';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';
import fs from 'fs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const app = new Koa();
// Use koa-compress middleware
app.use(compress());
// Serve static files with '/admin' prefix
app.use(mount('/', serve(path.join(__dirname, './buildc'))));

// Fallback route
app.use(async (ctx) => {
  ctx.type = 'html';
  ctx.body = fs.createReadStream(path.join(__dirname, './buildc/index.html'));
});

app.listen(5000, () => {
  console.log('Server is running on http://localhost:5000');
});