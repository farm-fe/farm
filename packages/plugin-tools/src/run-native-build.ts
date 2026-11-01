import { spawn } from 'node:child_process';

export function runNativeBuild(cliPath: string, args: string[], cwd: string) {
  return new Promise<void>((resolve, reject) => {
    const child = spawn(process.execPath, [cliPath, 'build', ...args], {
      cwd,
      stdio: 'inherit',
      shell: false
    });
    child.once('error', reject);
    child.once('close', (code, signal) => {
      if (code === 0) resolve();
      else
        reject(
          new Error(`Native build failed: code=${code}, signal=${signal}`)
        );
    });
  });
}
