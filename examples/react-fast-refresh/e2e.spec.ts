import { startAndTest, editFile, expect } from '../../e2e/index.ts';
import type { SpecContext, Page } from '../../e2e/index.ts';
import type { ElementHandle } from 'playwright-chromium';
import path, { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

const delay = (ms: number): Promise<void> => new Promise((r) => setTimeout(r, ms));

function waitMatchConsole(
  page: Page,
  text: string,
  timeout = 10_000
): Promise<void> {
  return new Promise((resolve, reject) => {
    let timer: NodeJS.Timeout | null = setTimeout(() => {
      reject(new Error('wait match console message timeout'));
    }, timeout);

    const handler = (message: any): void => {
      if (message.text().includes(text)) {
        if (timer) {
          clearTimeout(timer);
          timer = null;
        }
        page.off('console', handler);
        resolve();
      }
    };

    page.on('console', handler);
  });
}

async function expectTestFileHmr(
  page: Page,
  element: ElementHandle<SVGElement | HTMLElement>,
  filename: string,
  originText: string,
  afterText: string
): Promise<void> {
  const matchUpdateMessage = `[Farm HMR] ${path.posix.normalize(filename)} updated`;
  const waitUpdatePromise = waitMatchConsole(page, matchUpdateMessage);
  const recover = await editFile(path.join(projectPath, filename), originText, afterText);

  try {
    await waitUpdatePromise;
    await delay(1000);
    const content = await element.textContent();
    expect(content).toContain(afterText);
  } finally {
    await recover?.();
  }
}

async function expectUpdateError(
  page: Page,
  filename: string,
  originText: string,
  errorText: string
): Promise<void> {
  const errorMessage = '[Farm HMR] Parse `src/index.tsx` failed.';
  const recover = await editFile(path.join(projectPath, filename), originText, errorText);
  try {
    await waitMatchConsole(page, errorMessage, 10_000);
  } finally {
    await recover?.();
  }
}

async function expectRecoverFromError(
  page: Page,
  element: ElementHandle<SVGElement | HTMLElement>,
  filename: string,
  errorText: string,
  recoverText: string
): Promise<void> {
  const matchUpdateMessage = 'recovered 123';
  const recover = await editFile(path.join(projectPath, filename), errorText, recoverText);
  try {
    await waitMatchConsole(page, matchUpdateMessage, 10_000);
    await delay(1000);
  } finally {
    await recover?.();
  }
}

export default async function (ctx: SpecContext): Promise<void> {
  await ctx.test('run start (HMR)', async () => {
    await startAndTest(projectPath, async (page) => {
      const root = (await page.$('#root'))!;
      expect(root).not.toBeNull();

      const content = await root?.textContent();
      expect(content).toContain('class component');
      expect(content).toContain('function component');

      await expectTestFileHmr(
        page,
        root,
        './src/components/ClassC.tsx',
        'class component',
        'class component update'
      );
      await delay(3000);

      await expectTestFileHmr(
        page,
        root,
        './src/components/FnC.tsx',
        'function component',
        'function component update'
      );
      await delay(1000);

      const indexTsxPath = './src/index.tsx';
      await expectUpdateError(page, indexTsxPath, 'const a = 123;', 'const a =');
      await delay(1000);

      await expectRecoverFromError(
        page,
        root,
        indexTsxPath,
        'const a =',
        'const a = 123;console.log(`recovered ${a}`);'
      );
    });
  });
}
