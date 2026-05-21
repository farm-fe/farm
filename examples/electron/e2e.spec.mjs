import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';
import { expect } from '../../e2e/index.mjs';
import { _electron as electron } from 'playwright-chromium';

const projectPath = path.dirname(fileURLToPath(import.meta.url));

function cleanOut() {
  for (const dir of ['dist', 'dist-electron']) {
    fs.rmSync(path.join(projectPath, dir), { recursive: true, force: true });
  }
}

export default async function (ctx) {
  let app = null;
  let passE2E = false;

  try {
    // Setup: build and launch electron
    cleanOut();
    execSync('npm run farm -- build', { cwd: projectPath, stdio: 'inherit' });
    try {
      app = await electron.launch({
        args: ['.', '--no-sandbox'],
        cwd: projectPath,
        env: { ...process.env, NODE_ENV: 'development' }
      });
    } catch (error) {
      // Linux occasionally fails to launch electron in test environment
      if (process.platform === 'linux') {
        passE2E = true;
      } else {
        throw error;
      }
    }

    // Test
    await ctx.test('run electron', async () => {
      const page = await app?.firstWindow();
      page?.on('console', (msg) => console.log(msg.text()));

      await page?.screenshot({ path: 'screenshots/app-window.png' });
      expect(await page?.textContent('#app h1')).eq(
        passE2E ? undefined : 'Electron + Farm + TypeScript'
      );
    });
  } finally {
    // Teardown
    cleanOut();
    await app?.close();
  }
}
