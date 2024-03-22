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
const serverPorts = new Set();

function getServerPort(): number {
  const getRandomPort = () => Math.floor(Math.random() * 1000 + 9100);
  let port = getRandomPort();

  while (serverPorts.has(port)) {
    port = getRandomPort();
  }

  serverPorts.add(port);
  return port;
}

const visitPage = async (
  path: string,
  examplePath: string,
  cb: (page: Page) => Promise<void>
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
      logger(`${examplePath} -> ${path}: ${msg.text()}`);
      // browserLogs.push(msg.text());
    });
    let resolve: (data: any) => void, reject: (e: Error) => void;
    const promise = new Promise((r, re) => {
      resolve = r;
      reject = re;
    });

    page?.on('pageerror', (error) => {
      logger(`${examplePath} -> ${path}: ${error}`, {
        color: 'red'
      });
      reject(error);
    });

    page?.on('load', async () => {
      cb(page)
        .then(() => {
          resolve(null);
        })
        .catch((e) => {
          logger(`test error: ${examplePath} start failed with error ${e}`, {
            color: 'red'
          });
          reject(e);
        })
        .finally(() => {
          console.log('close page');
          page.close();
        });
    });

    await page?.goto(path);
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
  await new Promise((resolve, reject) => {
    logger(`Executing node ${cliBinPath} ${command} in ${examplePath}`);
    const child = execa('node', [cliBinPath, command], {
      cwd: examplePath,
      stdin: 'pipe',
      encoding: 'utf8',
      env: {
        BROWSER: 'none',
        NO_COLOR: 'true',
        FARM_DEFAULT_SERVER_PORT: String(port)
      }
    });

    let pagePath: null | string;
    let result = Buffer.alloc(0);
    const urlRegex =
      /((http|https):\/\/(localhost|\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}))(:\d+)?(\/[^\s]*)?/g;
    child.stdout?.on('data', async (chunk) => {
      result = Buffer.concat([result, chunk]); // 将 chunk 添加到 result 中
      if (pagePath) return;
      const res = result.toString();
      const replacer = res.replace(/\n/g, '');

      const matches = replacer.match(urlRegex);
      pagePath = matches && (matches[1] || matches[0]);

      if (pagePath) {
        try {
          await visitPage(pagePath, examplePath, cb);
          resolve(pagePath);
        } catch (e) {
          console.log('visit page error: ', e);
          reject(e);
        } finally {
          if (!child.killed) {
            child.kill();
          }
        }
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
        logger(`${examplePath} ${command} failed with code ${code}`, {
          color: 'red'
        });
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
