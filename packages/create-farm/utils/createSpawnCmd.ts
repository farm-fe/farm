import { spawn, StdioOptions } from 'child_process';

function createSpawnCmd(dest: string, stdio: StdioOptions = 'inherit') {
  return function (cmd: string, args?: string[]): Promise<unknown> {
    const ls = spawn(cmd, args, {
      cwd: dest,
      stdio: stdio,
      shell: true,
    });
    return new Promise((resolve, reject) => {
      ls.on('close', (code: any) => {
        code === 0 ? resolve(true) : reject(false);
      });
    }).catch((err) => {
      console.log(err);
    });
  };
}

export default createSpawnCmd;
