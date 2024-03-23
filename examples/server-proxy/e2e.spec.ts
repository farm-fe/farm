import { expect, test } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import { basename, dirname } from 'path';
import { fileURLToPath } from 'url';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

test(`e2e tests - ${name}`, async () => {
  const runTest = (command?: 'start' | 'preview') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        const app = await page.$('#app');
        console.log(await page.innerHTML('body'));
        expect(await app?.innerHTML()).toBe('app');
      },
      command
    );
  await launchServer();
  try {
    await runTest();
    await runTest('preview');
  } catch (e) {
    throw e;
  } finally {
    await closeServer();
  }
});

let server;

async function launchServer() {
  const { default: express } = await import('express');
  const app = express();
  app.use(async (req, res, next) => {
    res.json({
      hello: 'world'
    });
  });
  return new Promise((r) => {
    server = app.listen(3000, () => {
      console.log('server up');
      r(null);
    });
  });
}

async function closeServer() {
  server.close();
}
