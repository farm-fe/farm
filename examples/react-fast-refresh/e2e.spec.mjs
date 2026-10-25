import { startAndTest, editFile, expect } from '../../e2e/index.mjs';
import path, { dirname } from 'path';
import { fileURLToPath } from 'url';

const projectPath = dirname(fileURLToPath(import.meta.url));
const timeout = 10_000;

async function waitForText(page, text, absent) {
  await page.waitForFunction(
    ({ text, absent }) => {
      const content = document.querySelector('#root')?.textContent ?? '';
      return content.includes(text) && (!absent || !content.includes(absent));
    },
    { text, absent },
    { timeout }
  );
}

async function expectTestFileHmr(page, element, filename, originText, afterText) {
  const recover = await editFile(path.join(projectPath, filename), originText, afterText);
  try {
    await waitForText(page, afterText);
    expect(await element.textContent()).toContain(afterText);
  } finally {
    await recover?.();
    await waitForText(page, originText, afterText);
  }
}

async function expectErrorAndRecovery(page, root) {
  const filename = path.join(projectPath, 'src/index.tsx');
  const original = 'const a = 123;';
  const invalid = 'const a =';
  const repaired = 'const a = 123;document.documentElement.dataset.hmrRecovered = String(a);';
  const recover = await editFile(filename, original, invalid);
  try {
    const overlay = await page.waitForSelector('farm-error-overlay', { state: 'visible', timeout });
    const message = await overlay.evaluate(element => element.shadowRoot?.textContent ?? element.textContent);
    expect(message.replaceAll('\\', '/')).toContain('src/index.tsx');
    expect(message).toContain('Parse');
    expect(await root.textContent()).toContain('function component');
    expect(await page.locator('[data-refresh-counter]').textContent()).toContain('count: 2');
    await editFile(filename, invalid, repaired);
    await page.waitForFunction(() => document.documentElement.dataset.hmrRecovered === '123', undefined, { timeout });
    await page.waitForSelector('farm-error-overlay', { state: 'detached', timeout });
    await waitForText(page, 'class component');
    await waitForText(page, 'function component');
  } finally {
    await recover?.();
  }
}

export default async function (ctx) {
  await ctx.test('run start (HMR)', async () => {
    await startAndTest(projectPath, async (page) => {
      const root = await page.$('#root');
      expect(root).not.toBeNull();
      expect(await root.textContent()).toContain('class component');
      expect(await root.textContent()).toContain('function component');
      await expectTestFileHmr(page, root, './src/components/ClassC.tsx', 'class component', 'class component update');
      const counter = page.locator('[data-refresh-counter]');
      await counter.click();
      await counter.click();
      expect(await counter.textContent()).toContain('count: 2');
      await expectTestFileHmr(page, root, './src/components/FnC.tsx', 'function component', 'function component update');
      expect(await counter.textContent()).toContain('count: 2');
      await expectErrorAndRecovery(page, root);
    });
  });
}
