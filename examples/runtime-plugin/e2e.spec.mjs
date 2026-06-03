import { execFile } from 'node:child_process';
import { dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { promisify } from 'node:util';

const execFileAsync = promisify(execFile);
const projectPath = dirname(fileURLToPath(import.meta.url));

export default async function (ctx) {
  await ctx.test('build verifies runtime plugin resource loading', async () => {
    await execFileAsync('npm', ['run', 'build'], {
      cwd: projectPath,
      timeout: 120_000
    });
  });
}
