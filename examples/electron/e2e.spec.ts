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
let passE2E = false;

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
  try {
    app = await electron.launch({
      args: ['.', '--no-sandbox'],
      cwd: projectPath,
      env: { ...process.env, NODE_ENV: 'development' },
    });
  } catch (error: any) {
    if (process.platform === 'linux') {
      // Error: electron.launch: Process failed to launch!
      // https://github.com/microsoft/playwright/issues/11932#issuecomment-1200164702

      // TODO: Currently, the e2e test of Electron is still in an experimental state 
      // https://github.com/microsoft/playwright/blob/v1.44.1/docs/src/api/class-electron.md#class-electron
      passE2E = true;
    } else {
      throw error;
    }
  }
});

test(`e2e tests - ${name}`, async () => {
  const page = await app?.firstWindow();
  page?.on('console', (msg) => console.log(msg.text()));
  
  await page?.screenshot({ path: 'screenshots/app-window.png' });
  expect(await page?.textContent('#app h1')).eq(passE2E
    ? undefined
    : 'Electron + Farm + TypeScript');
});
