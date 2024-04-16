const path = require('path');
const fsp = require('fs/promises');
const express = require('express');

function resolve(p) {
  return path.resolve(__dirname, p);
}

async function createServer() {
  let app = express();

  app.use(express.static(resolve('build')));

  app.use('/', async (req, res) => {
    let url = req.originalUrl;

    try {
      let template;
      let render;

      template = await fsp.readFile(resolve('build/index_client.html'), 'utf8');
      render = require(resolve('dist/index.js'));

      const renderedHtml = render(url);
      console.log(renderedHtml);

      let html = template.replace('{app-html-to-replace}', renderedHtml);
      console.log(template.includes('{app-html-to-replace}'));
      console.log(html.includes('{app-html-to-replace}'));

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
