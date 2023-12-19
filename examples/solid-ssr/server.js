import path from 'path';
import fsp from 'fs/promises';
import express from 'express';
import { generateHydrationScript } from 'solid-js/web';

function resolve(p) {
  return path.resolve(p);
}

async function createServer() {
  let app = express();

  app.use(express.static(resolve('build')));

  app.use('/', async (req, res) => {
    let url = req.originalUrl;

    try {
      let template = await fsp.readFile(resolve('build/client.html'), 'utf8');
      let render = (await import(resolve('dist/index.js'))).default;

      const renderedHtml = render(url);

      let html = template
        .replace('<div>app-html-to-replace</div>', renderedHtml)
        .replace('</head>', generateHydrationScript());
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
  app.listen(3000, () => {
    console.log('HTTP server is running at http://localhost:3000');
  });
});
