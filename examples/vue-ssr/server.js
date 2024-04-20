import path from 'path';
import fsp from 'fs/promises';
import express from 'express';
import { pathToFileURL } from 'url';

function resolve(p) {
  return path.resolve(p);
}

async function createServer() {
  let app = express();

  app.use(express.static(resolve('build')));

  app.use('/', async (req, res) => {
    let url = req.originalUrl;

    try {
      let template;
      let render;

      template = await fsp.readFile(resolve('build/client.html'), 'utf8');
      const serverEntry = resolve('dist/index.js');
      render = (await import(process.platform === 'win32' ? pathToFileURL(serverEntry) : serverEntry)).default;

      const renderedHtml = await render(url);
      console.log(renderedHtml);

      let html = template.replace(
        '<div>app-html-to-replace</div>',
        renderedHtml
      );
      console.log(template.includes('<div>app-html-to-replace</div>'));
      console.log(html.includes('<div>app-html-to-replace</div>'));

      res.setHeader('Content-Type', 'text/html');
      return res.status(200).end(html);
    } catch (error) {
      console.log(error.stack);
      res.status(500).end(error.stack);
    }
  });

  return app;
}

createServer().then((app) => {
  const port = process.env.FARM_DEFAULT_SERVER_PORT || 3000;
  app.listen(port, () => {
    console.log('HTTP server is running at http://localhost:' + port);
  });
});
