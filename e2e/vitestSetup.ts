import { chromium, type Page } from 'playwright-chromium';
import { join } from 'path';
import { logger } from './utils.js';
import { inject, onTestFinished } from 'vitest';
import { execa } from 'execa';
import { existsSync } from 'fs';

// export const browserLogs: string[] = [];
// export const browserErrors: Error[] = [];
export const concurrencyLimit = 50;
export const pageMap = new Map<string, Page>();

const globalVar = globalThis as any;
globalVar.CURRENT_PORT = 9100;

function getServerPort(): number {
  const incPort = () => {
    globalVar.CURRENT_PORT += 10;
    console.log('generate port', globalVar.CURRENT_PORT);
    return globalVar.CURRENT_PORT;
  };
  return incPort();
}

const visitPage = async (
  path: string,
  examplePath: string,
  rawCb: (page: Page) => Promise<void>,
  command: string
) => {
  if (!path) return;
  // eslint-disable-next-line @typescript-eslint/ban-ts-comment
  // @ts-ignore
  const wsEndpoint = inject('wsEndpoint');
  if (!wsEndpoint) {
    throw new Error('wsEndpoint not found');
  }

  // make sure rawCb is called only once
  const cb = (function () {
    let called = false;
    return async (page: Page) => {
      if (called) return;
      called = true;
      return rawCb(page);
    };
  })();

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

    cb(page)
      .then(() => {
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

const getFarmCLIBinPath = (examplePath: string) => {
  try {
    const binPath = join('node_modules', '@farmfe', 'cli', 'bin', 'farm.mjs');
    const fullBinPath = join(examplePath, binPath);

    if (existsSync(fullBinPath)) {
      return binPath;
    }
    return '';
  } catch (error) {
    // console.error(' read json failed', error);
    return '';
  }
};

export const startProjectAndTest = async (
  examplePath: string,
  cb: (page: Page) => Promise<void>,
  command = 'start'
) => {
  // using bin path to spawn child process to avoid port conflicts issue
  const cliBinPath = getFarmCLIBinPath(examplePath);

  if (!cliBinPath) {
    throw new Error(`example ${examplePath} does not install @farmfe/cli`);
  }
  const port = getServerPort();
  logger(`Executing node ${cliBinPath} ${command} in ${examplePath}`);
  const child = execa('node', [cliBinPath, command], {
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
      const replacer = res.replace(/\n/g, '');

      const matches = replacer.match(urlRegex);
      const pagePath = matches && (matches[1] || matches[0]);

      if (pagePath) {
        resolve(pagePath);
      }
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
