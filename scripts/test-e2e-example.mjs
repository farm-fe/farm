import { buildCli, buildCoreCjs, buildJsPlugins } from './build.mjs';
import { chromium } from 'playwright-chromium'
import fs from "fs"
import { join } from 'node:path'
import { concurrentMap } from "./utils.mjs"
import { execa } from "execa"
import { logger } from './logger.mjs'

const browserLogs = []
const browserErrors = []
let browser;
const concurrencyLimit = 50


async function setup() {
  browser = await chromium.launch({
    headless: true,
  });
}

const visite = async (path) => {
  if (!path) return;
  let page = await browser.newPage();
  logger(`open the page: ${path}`);
  try {
    page.on('console', (msg) => {
      logger(`${path}:${msg.text()}`);
      browserLogs.push(msg.text())
    });

    page.on('pageerror', (error) => {
      logger(`${path}:${error}`, {
        color: "red"
      });
      browserErrors.push(error)
    });
    await page.goto(path);
  } catch (e) {
    await page.close()
    throw e
  }
}

let exampleHasStartCommond = (examplePath) => {
  try {
    const packageJson = JSON.parse(fs.readFileSync(join(examplePath, "package.json"), 'utf8'));

    return packageJson && packageJson.scripts && packageJson.scripts.start
  } catch (error) {
    console.error('无法解析 package.json 文件:', error);
    return false;
  }
}

let startProjectAndVisite = (examplePath) => {
  if (!exampleHasStartCommond(examplePath)) return;
  const { stdout } = execa('npm', ['run', 'start'], {
    cwd: examplePath,
    stdout: 'pipe',
    env: {
      "BROWSER": "none"
    }
  });
  let pagePath;
  let result = '';
  const urlRegex = /http(s|):\/\/\S*?\//;
  stdout && stdout.on("data", async (res) => {
    if (pagePath) return;
    result += res.toString();
    let matches = result.replace(/\n/g, '').match(urlRegex);
    pagePath = matches && matches[0];
    pagePath && await visite(pagePath);
  });
}


async function startE2eTest() {
  setup();
  const examples = fs.readdirSync(("./examples"));
  await Promise.all(concurrentMap(examples, concurrencyLimit, async (example) => {
    console.log(example, "example");
    const examplePath = join('./examples', example)
    if (fs.statSync(examplePath).isDirectory()) {
      startProjectAndVisite(examplePath);
    }
  }));
}

async function main() {
  console.log('Building CLI...');
  await buildCli();
  console.log('Building core CJS...');
  await buildCoreCjs();
  console.log('Building JS plugins...');
  await buildJsPlugins();
  console.log('E2e testing for examples');
  await startE2eTest();
}

main()


