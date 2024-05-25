import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';
import { afterAll, beforeAll, expect, test } from 'vitest';
import {
  type ElectronApplication,
  _electron as electron,
} from 'playwright-chromium';

const name = path.basename(import.meta.url);
const projectPath = path.dirname(fileURLToPath(import.meta.url));
let app: ElectronApplication | null = null;

function cleanOut() {
  const outDirs = ['dist', 'dist-electron']
  for (const dir of outDirs) {
    fs.rmSync(path.join(projectPath, dir), { recursive: true, force: true });
  }
}

afterAll(async () => {
  cleanOut();
  app?.close();
  app = null;
});

beforeAll(async () => {
  cleanOut();
  execSync('npm run farm -- build', { cwd: projectPath, stdio: 'inherit' });
  app = await electron.launch({
    args: ['.', '--no-sandbox'],
    cwd: projectPath,
    // https://github.com/microsoft/playwright/issues/11932#issuecomment-1200164702
    env: { ...process.env, NODE_ENV: 'development' },
  });
});

test(`e2e tests - ${name}`, async () => {
  const page = await app?.firstWindow();

  await page?.screenshot({ path: 'screenshots/app-window.png' });
  expect(await page?.textContent('#app h1')).toBe('Electron + Farm + TypeScript');
});
