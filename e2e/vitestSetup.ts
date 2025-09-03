import { chromium, type Page } from 'playwright-chromium';
import { logger } from './utils.js';
import { inject, onTestFinished } from 'vitest';
import { execa } from 'execa';

// export const browserLogs: string[] = [];
// export const browserErrors: Error[] = [];
export const concurrencyLimit = 50;

function getServerPort(): Promise<number> {
  // retry 3 times
  let retryCount = 0;

  return fetch('http://127.0.0.1:12306/port')
    .then((r) => r.text())
    .then(Number)
    .catch(async () => {
      if (retryCount < 3) {
        await new Promise((resolve) => setTimeout(resolve, 500));
        retryCount++;
        return getServerPort();
      }
      throw new Error('get server port failed');
    });
}

const visitPage = async (
  path: string,
  examplePath: string,
  cb: (page: Page) => Promise<void>,
  command: string
) => {
  if (!path) return;
  // @ts-ignore
  const wsEndpoint = inject('wsEndpoint');
  if (!wsEndpoint) {
    throw new Error('wsEndpoint not found');
  }

  const browser = await chromium.connect(wsEndpoint);
  const page = await browser?.newPage();
  page.on('requestfailed', (req) => {
    logger(`request failed ${path} ${examplePath}: ${req.url()} ${req.failure()?.errorText} ${req}`, {
      color: 'red'
    });
  });
  logger(`open the page: ${path} ${examplePath}`);
  try {
    page?.on('console', (msg) => {
      const lowerCaseMsg = msg.text().toLocaleLowerCase();

      if (msg.type() === 'error' && !lowerCaseMsg.includes('warn') && !lowerCaseMsg.includes('warning') && !/Parse `.+` failed/.test(msg.text())) {
        logger(`command ${command} ${examplePath} -> ${path}: ${msg.text()}`, {
          color: 'red'
        });
        reject(new Error(msg.text()));
      } else {
        logger(`command ${command} ${examplePath} -> ${path}: ${msg.text()}`);
      }
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
  //   throw new Error(`example ${examplePath} does not install farm`);
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
      result = Buffer.concat([result, chunk]);
      const res = result.toString();
      const replacer = res.replace(/\n/g, ' ');

      const matches = replacer.match(urlRegex);
      const pagePath = matches?.[0]; // use localhost for test

      if (pagePath) {
        resolve(pagePath);
      }
    });

    child.stderr.on('data', (chunk) => {
      logger(chunk.toString(), { color: 'red' });
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
        child.kill(0);
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
      child.kill(0);
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
      }, 60000);
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
