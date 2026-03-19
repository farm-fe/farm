import express from 'express';
import { Server } from '@farmfe/core';

const app = express();
const port = 3000;

const farmServer = await Server.createServer({
  configPath: './farm.config.ts'
});
const moduleRunner = await farmServer.createModuleRunner({ hmr: true });

app.get('/api/ping', (_, res) => {
  res.json({ ok: true, from: 'express-api' });
});

app.get('/api/runner', async (_, res) => {
  try {
    const mod = await moduleRunner.import('/src/entry-server.mjs');
    res.json({
      ok: true,
      from: 'module-runner',
      value: mod.value,
      at: mod.now()
    });
  } catch (error) {
    res.status(500).json({
      ok: false,
      message: error instanceof Error ? error.message : String(error)
    });
  }
});

app.use(farmServer.middlewares);

const httpServer = app.listen(port, () => {
  console.log(`middleware host: http://localhost:${port}`);
  console.log('hmr ws port: 9801');
  console.log('try GET /api/ping, GET /api/runner and open /');
});

const close = async () => {
  await farmServer.close();
  await new Promise((resolve, reject) => {
    httpServer.close((error) => (error ? reject(error) : resolve(undefined)));
  });
};

process.on('SIGINT', async () => {
  await close();
  process.exit(0);
});

process.on('SIGTERM', async () => {
  await close();
  process.exit(0);
});
