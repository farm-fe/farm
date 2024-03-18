import { chromium, type Browser, type Page } from 'playwright-chromium'
import { join } from 'path'
import { readFileSync, readdirSync, statSync } from 'node:fs'
import { concurrentMap, logger } from './utils'
import { execa } from "execa"
import { beforeAll, inject } from 'vitest'
import type { File } from 'vitest'


export const browserLogs: string[] = []
export const browserErrors: Error[] = []
export const concurrencyLimit = 50
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
    await page?.close();
    throw e
  }
}

let exampleHasStartCommond = (examplePath) => {
  try {
    const packageJson = JSON.parse(readFileSync(join(examplePath, "package.json"), 'utf8'));

    return packageJson && packageJson.scripts && packageJson.scripts.start
      && packageJson.scripts.start.includes("start");
  } catch (error) {
    // console.error(' read json failed', error);
    return false;
  }
}

let startProjectAndVisite = async (examplePath: string) => {
  if (!exampleHasStartCommond(examplePath)) {
    return
  };
  await new Promise(async (resolve) => {
    const { stdout,stderr } = execa('npm', ['run', 'start'], {
      cwd: examplePath,
      stdin: 'pipe',
      encoding: 'utf8',
      env: {
        BROWSER: "none",
        NO_COLOR: "true"
      },
    });
    let pagePath;
    let result = Buffer.alloc(0);
    const urlRegex = /((http|https):\/\/(localhost|\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}))(:\d+)?(\/[^\s]*)?/g;
    
    stdout.on(("data"), async (chunk) => {
      result = Buffer.concat([result, chunk]); // 将 chunk 添加到 result 中
      if (pagePath) return;
      let res = result.toString();
      let replacer = res.replace(/\n/g, '')

      let matches = replacer.match(urlRegex);
      pagePath = matches && (matches[1] || matches[0]);
      
      if (pagePath) {
        await visite(pagePath, examplePath);
        resolve(pagePath);
      }
    });

    stdout.on("end", () => {
      resolve(null);
    });
    
    stdout.on("error", (error)=> {
      console.log(error);
      resolve(error);
    });

    stderr.on('close', (error)=> {
      console.log(error);
      resolve(error);
    });
  })
}

async function startTest() {
  const examples = readdirSync(("./examples"));
  await Promise.all(concurrentMap(examples, concurrencyLimit, async (example) => {
    const examplePath = join('./examples', example)

    if (statSync(examplePath).isDirectory()) {
      return await startProjectAndVisite(examplePath);
    }
    return;
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
    // startTest();
  }
})
