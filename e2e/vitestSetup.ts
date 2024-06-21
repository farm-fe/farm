import { chromium, type Page } from 'playwright-chromium';
import { logger } from './utils.js';
import { inject, onTestFinished, beforeEach, beforeAll } from 'vitest';
import { execa } from 'execa';
import { mkdir, writeFile, readFile, unlink } from 'node:fs/promises';
import { createWriteStream, existsSync } from 'node:fs'
import path from 'node:path';

// export const browserLogs: string[] = [];
// export const browserErrors: Error[] = [];
export const concurrencyLimit = 50;
export const pageMap = new Map<string, Page>();

const DEFAULT_PORT = 9100;
const PORT_RECORD = 'port-record.json';
const PORT_LOCK = 'port-record.lock';

const delay = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

async function getServerPort(): Promise<number> {
  const basedir = path.join(process.cwd(), 'node_modules/.farm/.test-e2e-port-lock');
  const filename = path.join(basedir, PORT_RECORD);
  const lockfile = path.join(basedir, PORT_LOCK);
  let count = 0;
  try {
    while(true) {
      if (count > 10) {
        // if timeout, it's maybe error file
        await unlink(lockfile);
      }
      if (!existsSync(PORT_LOCK)) {
        await writeFile(lockfile, '');
        count = 0;
      }else {
        count++;
        await delay(30);
        continue;
      }

      if (!existsSync(filename)) {
        await mkdir(path.dirname(filename), { recursive: true })
        await writeFile(filename, DEFAULT_PORT.toString());
      }

      const port = Number(await readFile(filename, 'utf-8'))

      await writeFile(filename, (port + 10).toString());

      return port;
    }

  } finally {
    await unlink(lockfile);
  }
}

const visitPage = async (
  path: string,
  examplePath: string,
  cb: (page: Page) => Promise<void>,
  command: string
) => {
  if (!path) return;
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  const wsEndpoint = inject('wsEndpoint');
  if (!wsEndpoint) {
    throw new Error('wsEndpoint not found');
  }

  const browser = await chromium.connect(wsEndpoint);
  const page = await browser?.newPage();
  page && pageMap.set(path, page);
  logger(`open the page: ${path} ${examplePath}`);
  try {
    page?.on('console', (msg) => {
      logger(`command ${command} ${examplePath} -> ${path}: ${msg.text()}`);
      // browserLogs.push(msg.text());
    });
    let resolve: (data: any) => void, reject: (e: Error) => void;
    const promise = new Promise((r, re) => {
      resolve = r;
      reject = re;
    });

    page?.on('pageerror', (error) => {
      logger(`command ${command} ${examplePath} -> ${path}: ${error}`, {
        color: 'red'
      });
      reject(error);
    });

    page?.on('load', async () => {
      console.log(command, 'page load');
    });

    await page?.goto(path);
    console.log('test page', examplePath);
    cb(page)
      .then(() => {
        console.log('test page success', examplePath);
        resolve(null);
      })
      .catch((e) => {
        logger(
          `command ${command} test error: ${examplePath} start failed with error ${e}`,
          {
            color: 'red'
          }
        );
        reject(e);
      })
      .finally(() => {
        console.log('test page finish', examplePath, 'close page');
        page?.close({
          reason: 'test finished',
          runBeforeUnload: false
        });
      });

    return promise;
  } catch (e) {
    await page?.close();
    throw e;
  }
};

export const startProjectAndTest = async (
  examplePath: string,
  cb: (page: Page) => Promise<void>,
  command = 'start'
) => {
  // // using bin path to spawn child process to avoid port conflicts issue
  // const cliBinPath = getFarmCLIBinPath(examplePath);

  // if (!cliBinPath) {
  //   throw new Error(`example ${examplePath} does not install @farmfe/cli`);
  // }
  const port = await getServerPort();
  logger(`Executing npm run ${command} in ${examplePath}`);
  const child = execa('npm', ['run', command], {
    cwd: examplePath,
    stdin: 'pipe',
    encoding: 'utf8',
    env: {
      BROWSER: 'none',
      NO_COLOR: 'true',
      FARM_DEFAULT_SERVER_PORT: String(port),
      FARM_DEFAULT_HMR_PORT: String(port)
    }
  });

  const pagePath = await new Promise<string>((resolve, reject) => {
    let result = Buffer.alloc(0);
    const urlRegex =
      /((http|https):\/\/(localhost|\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}))(:\d+)?(\/[^\s]*)?/g;
    child.stdout?.on('data', async (chunk) => {
      result = Buffer.concat([result, chunk]); // 将 chunk 添加到 result 中
      const res = result.toString();
      const replacer = res.replace(/\n/g, ' ');

      const matches = replacer.match(urlRegex);
      const pagePath = matches && (matches[1] || matches[0]);

      if (pagePath) {
        resolve(pagePath);
      }
    });

    child.stderr.on('data', chunk => {
      logger(chunk.toString(), { color: 'red' });
    })

    child.on('error', (error) => {
      logger(
        `child process error: ${examplePath} ${command} failed with error ${error}`,
        {
          color: 'red'
        }
      );
      reject(
        `child process error: ${examplePath} ${command} failed with error ${error}`
      );
    });

    child.on('exit', (code) => {
      if (code) {
        logger(
          `${examplePath} ${command} failed with code ${code}. ${result.toString(
            'utf-8'
          )}`,
          {
            color: 'red'
          }
        );
        reject(new Error(`${examplePath} ${command} failed with code ${code}`));
      }
    });

    onTestFinished(() => {
      logger('try kill child process: ' + child.pid);
      logger('current process id: ' + process.pid);
      if (!child.killed) {
        child.kill();
      }
    });
  });

  try {
    await visitPage(pagePath, examplePath, cb, command);
  } catch (e) {
    console.log('visit page error: ', e);
    throw e;
  } finally {
    if (!child.killed) {
      child.kill();
    }
  }
};

export const watchProjectAndTest = async (
  examplePath: string,
  cb: (log: string, done: () => void) => Promise<void>,
  command = 'start'
) => {
  const port = getServerPort();
  logger(`Executing npm run ${command} in ${examplePath}`);
  const child = execa('npm', ['run', command], {
    cwd: examplePath,
    stdin: 'pipe',
    encoding: 'utf8',
    env: {
      BROWSER: 'none',
      NO_COLOR: 'true',
      FARM_DEFAULT_SERVER_PORT: String(port),
      FARM_DEFAULT_HMR_PORT: String(port)
    }
  });

  return new Promise((resolve, reject) => {
    let result = Buffer.alloc(0);
    child.stdout?.on('data', async (chunk) => {
      result = Buffer.concat([result, chunk]); // 将 chunk 添加到 result 中
      const res = result.toString();
      setTimeout(() => {
        reject(new Error('timeout'));
      }, 10000);
      cb(res, () => resolve(null));
    });

    child.on('error', (error) => {
      logger(
        `child process error: ${examplePath} ${command} failed with error ${error}`,
        {
          color: 'red'
        }
      );
      reject(
        `child process error: ${examplePath} ${command} failed with error ${error}`
      );
    });

    child.on('exit', (code) => {
      if (code) {
        logger(
          `${examplePath} ${command} failed with code ${code}. ${result.toString(
            'utf-8'
          )}`,
          {
            color: 'red'
          }
        );
        reject(new Error(`${examplePath} ${command} failed with code ${code}`));
      }
    });

    onTestFinished(() => {
      logger('try kill child process: ' + child.pid);
      logger('current process id: ' + process.pid);
      if (!child.killed) {
        child.kill();
      }
    });
  });
};
