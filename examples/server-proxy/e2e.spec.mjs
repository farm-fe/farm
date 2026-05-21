import { startAndTest, expect } from '../../e2e/index.mjs';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

let server;

async function launchServer() {
  const { default: express } = await import('express');
  const app = express();
  app.use((_req, res) => {
    res.json({ hello: 'world' });
  });

  return new Promise((resolve, reject) => {
    const timer = setTimeout(() => {
      reject(new Error('listen port 3000 timeout'));
    }, 3000);

    server = app.listen(3000, () => {
      clearTimeout(timer);
      resolve();
    });
  });
}

async function closeServer() {
  return new Promise((resolve) => {
    server?.close(() => resolve());
  });
}

export default async function (ctx) {
  const runTest = (command) =>
    startAndTest(
      projectPath,
      async (page) => {
        const app = await page.$('#app');
        expect(await app?.innerHTML()).toBe('app');
      },
      command
    );

  await launchServer();
  try {
    await ctx.test('run start', () => runTest());
    await ctx.test('run preview', () => runTest('preview'));
  } finally {
    await closeServer();
  }
}
