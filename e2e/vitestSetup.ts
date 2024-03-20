import { chromium, type Page } from 'playwright-chromium';
import { join } from 'path';
import { readFileSync } from 'node:fs';
import { logger } from './utils';
import { inject } from 'vitest';

export const browserLogs: string[] = [];
export const browserErrors: Error[] = [];
export const concurrencyLimit = 50;
export const pageMap = new Map<String, Page>();

const visitPage = async (
  path: string,
  examplePath: string,
  cb: (page: Page) => Promise<void>
) => {
  if (!path) return;
  // @ts-ignore
  const wsEndpoint = inject('wsEndpoint');
  if (!wsEndpoint) {
    throw new Error('wsEndpoint not found');
  }

  let browser = await chromium.connect(wsEndpoint);
  let page = await browser?.newPage();
  page && pageMap.set(path, page);
  logger(`open the page: ${path} ${examplePath}`);
  try {
    page?.on('console', (msg) => {
      logger(`${path}: ${msg.text()}`);
      browserLogs.push(msg.text());
    });

    page?.on('pageerror', (error) => {
      logger(`${path}: ${error}`, {
        color: 'red'
      });
      browserErrors.push(error);
    });

    const promise = new Promise((resolve, reject) => {
      page?.on('load', async () => {
        cb(page)
          .then(() => {
            resolve(null);
          })
          .catch((e) => {
            reject(e);
          });
      });
    });

    await page?.goto(path);
    return promise;
  } catch (e) {
    await page?.close();
    throw e;
  }
};

let exampleHasStartCommand = (examplePath: string) => {
  try {
    const packageJson = JSON.parse(
      readFileSync(join(examplePath, 'package.json'), 'utf8')
    );

    return (
      packageJson &&
      packageJson.scripts &&
      packageJson.scripts.start &&
      packageJson.scripts.start.includes('start')
    );
  } catch (error) {
    // console.error(' read json failed', error);
    return false;
  }
};

export const startProjectAndTest = async (
  examplePath: string,
  cb: (page: Page) => Promise<void>
) => {
  if (!exampleHasStartCommand(examplePath)) {
    return;
  }
  await new Promise(async (resolve, reject) => {
    const { execa } = await import('execa');

    const { stdout, stderr, kill, on } = execa('npm', ['run', 'start'], {
      cwd: examplePath,
      stdin: 'pipe',
      encoding: 'utf8',
      env: {
        BROWSER: 'none',
        NO_COLOR: 'true'
      }
    });
    let pagePath: null | string;
    let result = Buffer.alloc(0);
    const urlRegex =
      /((http|https):\/\/(localhost|\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}))(:\d+)?(\/[^\s]*)?/g;

    stdout?.on('data', async (chunk) => {
      result = Buffer.concat([result, chunk]); // 将 chunk 添加到 result 中
      if (pagePath) return;
      let res = result.toString();
      let replacer = res.replace(/\n/g, '');

      let matches = replacer.match(urlRegex);
      pagePath = matches && (matches[1] || matches[0]);

      if (pagePath) {
        try {
          await visitPage(pagePath, examplePath, cb);
          resolve(pagePath);
        } finally {
          kill();
        }
      }
    });

    on('error', (error) => {
      reject(error);
    });

    on('exit', (code) => {
      if (code !== 0) {
        reject(new Error(`start failed with code ${code}`));
      }
    });
  });
};
