import { startAndTest, expect } from '../../e2e/index.ts';
import type { SpecContext } from '../../e2e/index.ts';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

let server: any;

async function launchServer(): Promise<void> {
  const { default: express } = await import('express');
  const app = express();
  app.use((_req: any, res: any) => {
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

async function closeServer(): Promise<void> {
  return new Promise((resolve) => {
    server?.close(() => resolve());
  });
}

export default async function (ctx: SpecContext): Promise<void> {
  const runTest = (command?: 'start' | 'preview') =>
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
