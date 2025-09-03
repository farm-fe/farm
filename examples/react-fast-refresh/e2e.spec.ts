import { test, expect, describe } from 'vitest';
import { startProjectAndTest } from '../../e2e/vitestSetup';
import path, { basename, dirname } from 'path';
import { fileURLToPath } from 'url';
import { editFile } from '../../e2e/utils';
import { ConsoleMessage, ElementHandle, Page } from 'playwright-chromium';

const name = basename(import.meta.url);
const projectPath = dirname(fileURLToPath(import.meta.url));

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

const waitMatchConsole = (page: Page, text: string, timeout = 10000) => new Promise((resolve, reject) => {
  let timer: NodeJS.Timeout | null = setTimeout(() => {
    reject('wait match console message timeout');
  }, timeout);

  let cleanTimer = () => {
    if (timer) {
      clearTimeout(timer);
      timer = null;;
    }
  }
  let handler = (message: ConsoleMessage) => {

    if(message.text().includes(text)) {
      cleanTimer();
      resolve(undefined);

      page.off('console', handler);
    };
  };


  page.on('console', handler);
})

async function expectTestFileHmr(page: Page, element: ElementHandle<SVGElement | HTMLElement>, filename: string, originText: string, afterText: string) {

  const matchUpdateMessage = `[Farm HMR] ${path.posix.normalize(filename)} updated`;

  const waitUpdatePromise = waitMatchConsole(page, matchUpdateMessage);

  const recover = await editFile(path.join(projectPath, filename), originText, afterText);

  try {
    await waitUpdatePromise;
    await delay(1000);
    expect((await element.textContent())).toContain(afterText);
  } finally {
    await recover?.()
  }
}

const expectUpdateError = async (page: Page, filename: string, originText: string, errorText: string) => {
  const errorMessage = `[Farm HMR] Parse \`src/index.tsx\` failed.`;
  const recover = await editFile(path.join(projectPath, filename), originText, errorText);

  try {
    await waitMatchConsole(page, errorMessage, 10000);
  } finally {
    await recover?.();
  }
};

const expectRecoverFromError = async (page: Page, element: ElementHandle<SVGElement | HTMLElement>, filename: string, errorText: string, recoverText: string) => {
  const matchUpdateMessage = `recovered 123`;
  const recover = await editFile(path.join(projectPath, filename), errorText, recoverText);

  try {
    await waitMatchConsole(page, matchUpdateMessage, 10000);
    await delay(1000);
  } finally {
    await recover?.();
  }
};

describe(`e2e tests - ${name}`, async () => {
  const runTest = (command: 'start' = 'start') =>
    startProjectAndTest(
      projectPath,
      async (page) => {
        const root = (await page.$('#root'))!;

        expect(root).not.toBeNull();

        const content = await root?.textContent();

        expect(content).toContain('class component');

        expect(content).toContain('function component');

        await expectTestFileHmr(page, root, './src/components/ClassC.tsx', 'class component', 'class component update');

        await delay(3000);

        await expectTestFileHmr(page, root, './src/components/FnC.tsx', 'function component', 'function component update');

        await delay(1000);
        // 1. update index.tsx with syntax error, expect error output
        const indexTsxPath = './src/index.tsx';
        await expectUpdateError(page, indexTsxPath, 'const a = 123;', 'const a ='); // introduce syntax error
        await delay(1000);
        // 2. recover index.tsx, expect page output latest update
        await expectRecoverFromError(page, root, indexTsxPath, "const a =", 'const a = 123;console.log(`recovered ${a}`);');
      },
      command
    );

  test(`exmaples ${name} run start`, async () => {
    await runTest();
  });
});
