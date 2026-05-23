import { startAndTest, editFile, expect } from '../../e2e/index.mjs';
import path, { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));

const delay = (ms) => new Promise((r) => setTimeout(r, ms));

function waitMatchConsole(page, text, timeout = 10_000) {
  return new Promise((resolve, reject) => {
    let timer = setTimeout(() => {
      reject(new Error('wait match console message timeout'));
    }, timeout);

    const handler = (message) => {
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

async function expectTestFileHmr(page, element, filename, originText, afterText) {
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

async function expectUpdateError(page, filename, originText, errorText) {
  const errorMessage = '[Farm HMR] Parse `src/index.tsx` failed.';
  const recover = await editFile(path.join(projectPath, filename), originText, errorText);
  try {
    await waitMatchConsole(page, errorMessage, 10_000);
  } finally {
    await recover?.();
  }
}

async function expectRecoverFromError(page, element, filename, errorText, recoverText) {
  const matchUpdateMessage = 'recovered 123';
  const recover = await editFile(path.join(projectPath, filename), errorText, recoverText);
  try {
    await waitMatchConsole(page, matchUpdateMessage, 10_000);
    await delay(1000);
  } finally {
    await recover?.();
  }
}

export default async function (ctx) {
  await ctx.test('run start (HMR)', async () => {
    await startAndTest(projectPath, async (page) => {
      const root = await page.$('#root');
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
