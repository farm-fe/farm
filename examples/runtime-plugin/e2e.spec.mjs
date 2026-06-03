import { execFile } from 'node:child_process';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  await ctx.test('build verifies runtime plugin resource loading', async () => {
    try {
      await execFileAsync('npm', ['run', 'build'], {
        cwd: projectPath,
        timeout: 120_000
      });
    } catch (error) {
      const stdout = error?.stdout ? `\nstdout:\n${error.stdout}` : '';
      const stderr = error?.stderr ? `\nstderr:\n${error.stderr}` : '';
      throw new Error(`runtime-plugin build failed.${stdout}${stderr}`);
    }
  });
}
