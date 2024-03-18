import { chromium, type Browser, type Page } from 'playwright-chromium'
import { join } from 'path'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { concurrentMap, logger } from './utils'
import { execa } from "execa"
import { beforeAll, inject } from 'vitest'
import type { File } from 'vitest'


const browserLogs: string[] = []
const browserErrors: Error[] = []
const concurrencyLimit = 50
export const pageMap = new Map<String, Page>()

const visite = async (path: string, examplePath) => {
  if (!path) return;
  // @ts-ignore
  const wsEndpoint = inject('wsEndpoint')
  if (!wsEndpoint) {
    throw new Error('wsEndpoint not found')
  }

  let browser = await chromium.connect(wsEndpoint)
  let page = await browser?.newPage();
  page && pageMap.set(path, page);
  logger(`open the page: ${path} ${examplePath}`);
  try {
    page?.on('console', (msg) => {
      logger(`${path}: ${msg.text()}`);
      browserLogs.push(msg.text())
    });

    page?.on('pageerror', (error) => {
      logger(`${path}: ${error}`, {
        color: "red"
      });
      browserErrors.push(error)
    });
    await page?.goto(path);
  } catch (e) {
    await page?.close()
    throw e
  }
}

let exampleHasStartCommond = (examplePath) => {
  try {
    const packageJson = JSON.parse(readFileSync(join(examplePath, "package.json"), 'utf8'));

    return packageJson && packageJson.scripts && packageJson.scripts.start
      && packageJson.scripts.start.includes("start");
  } catch (error) {
    console.error(' read json failed', error);
    return false;
  }
}

let startProjectAndVisite = async (examplePath: string) => {
  if (!exampleHasStartCommond(examplePath)) {
    return
  };
  await new Promise(async (resolve, reject) => {
    const { stdout } = execa('npm', ['run', 'start'], {
      cwd: examplePath,
      stdin: 'pipe',
      env: {
        BROWSER: "none",
        encoding: 'Buffer',
      },
      maxBuffer: 100 * 1024 * 1024
    });
    let pagePath;
    let result = '';
    const urlRegex = /(http|https):\/\/((\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}))(:\d+)?(\/[^\s]*)?$/g;
    stdout && stdout.on("data", async (res) => {
      if (pagePath) return;
      result += res.toString('utf8');
      let replacer = result.replace(/\n/g, '')
      console.log(examplePath,replacer, 9999);

      let matches = replacer.match(urlRegex);
      pagePath = matches && matches[0];
      
      if (pagePath) {
        await visite(pagePath, examplePath);
        resolve(pagePath);
      }
    });
  })
}

async function startTest() {
  const examples = readdirSync(("./examples"));
  // [examples[1], examples[0]
  await Promise.all(concurrentMap(examples.slice(1, 10), concurrencyLimit, async (example) => {
    const examplePath = join('./examples', example)

    if (statSync(examplePath).isDirectory()) {
      return await startProjectAndVisite(examplePath);
    }
  })).catch((e) => {
    console.log(e)
  });
}

beforeAll(async (s) => {
  const suite = s as File
  if (
    !suite.filepath.includes("examples")
  ) {
    return
  }
  await startTest()

  return async () => {
    startTest();
  }
})
