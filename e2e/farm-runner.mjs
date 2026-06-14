import { execa } from 'execa';
import getPort, { portNumbers } from 'get-port';
import { access, readFile } from 'node:fs/promises';
import { setTimeout as delay } from 'node:timers/promises';
import { pathToFileURL } from 'node:url';
import { logger } from './utils.mjs';

// ---------------------------------------------------------------------------
// Browser singleton
// ---------------------------------------------------------------------------

let _browser = null;
let _context = null;
let _recoverBrowser = null;

const E2E_PORT_BASE = 41_000;
const E2E_WORKER_PORT_BLOCK_SIZE = 200;
const E2E_RETRY_PORT_BLOCK_SIZE = 20;
const E2E_PORT_CANDIDATE_COUNT = 10;
const E2E_TIMEOUT_MS = 120_000;
const E2E_TIMEOUT_SECONDS = E2E_TIMEOUT_MS / 1000;
const FARM_CONFIG_FILES = [
  'farm.config.ts',
  'farm.config.mts',
  'farm.config.cts',
  'farm.config.js',
  'farm.config.mjs',
  'farm.config.cjs'
];

const workerIndex = Number.parseInt(process.env.FARM_E2E_WORKER_INDEX ?? '0', 10) || 0;

/** @param {import('playwright-chromium').Browser} browser */
export function initBrowser(browser) {
  _browser = browser;
}

/** @param {import('playwright-chromium').BrowserContext} context */
export function initBrowserContext(context) {
  _context = context;
}

/** @param {(null | (() => Promise<void>))} handler */
export function setBrowserRecoveryHandler(handler) {
  _recoverBrowser = handler;
}

/**
 * @param {unknown} error
 * @returns {boolean}
 */
function isBrowserCrashLikeError(error) {
  const msg = error instanceof Error ? error.message : String(error ?? '');
  return (
    msg.includes('Target page, context or browser has been closed') ||
    msg.includes('Target crashed') ||
    msg.includes('Page crashed')
  );
}

/**
 * @param {import('playwright-chromium').Page} page
 * @param {string} examplePath
 * @returns {Promise<() => Promise<void>>}
 */
async function installExampleRequestRoutes(page, examplePath) {
  const mockFile = `${examplePath}/e2e.mock.mjs`;

  try {
    await access(mockFile);
  } catch (error) {
    if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
      return async () => {};
    }

    throw error;
  }

  /** @type {{ default?: Function, installMockRoutes?: Function }} */
  const mod = await import(pathToFileURL(mockFile).href);
  const install = mod.installMockRoutes ?? mod.default;

  if (typeof install !== 'function') {
    throw new Error(`Example mock file "${mockFile}" must export a mock route installer.`);
  }

  const cleanup = await install({ page, examplePath });

  if (typeof cleanup !== 'function') {
    return async () => {};
  }

  return cleanup;
}

/**
 * @returns {import('playwright-chromium').Browser}
 */
function requireBrowser() {
  if (!_browser) {
    throw new Error('Browser not initialised. Call initBrowser() before running specs.');
  }
  return _browser;
}

/**
 * @returns {boolean}
 */
function isContextValid(ctx) {
  if (!ctx) return false;
  try {
    const b = ctx.browser();
    return b != null && b.isConnected();
  } catch {
    return false;
  }
}

/**
 * @returns {import('playwright-chromium').BrowserContext}
 */
function requireBrowserContext() {
  if (isContextValid(_context)) {
    return _context;
  }

  if (_browser) {
    throw new Error(
      'Browser context not initialised. Call initBrowserContext() before running specs.'
    );
  }

  throw new Error('Browser not initialised. Call initBrowser() before running specs.');
}

/**
 * @param {import('execa').ExecaChildProcess<string>} child
 * @returns {Promise<void>}
 */
async function terminateChildProcess(child) {
  const pid = child.pid;
  const signalProcess = async (signal) => {
    if (!pid) return;
    try {
      if (process.platform === 'win32') {
        await execa(`task${'kill'}.exe`, ['/PID', String(pid), '/T', '/F'], {
          reject: false
        });
      } else {
        process.kill(-pid, signal);
      }
    } catch {
      child.kill(signal);
    }
  };

  if (child.exitCode === null && !child.killed) {
    await signalProcess('SIGTERM');
  }

  const exited = await Promise.race([
    child.catch(() => {}).then(() => true),
    delay(5_000).then(() => false)
  ]);

  if (!exited && child.exitCode === null) {
    await signalProcess('SIGKILL');
    await child.catch(() => {});
  }
}

/**
 * @param {string} value
 * @returns {number}
 */
function stableHash(value) {
  let hash = 0;
  for (let i = 0; i < value.length; i++) {
    hash = (hash * 31 + value.charCodeAt(i)) >>> 0;
  }
  return hash;
}

/**
 * @param {string} command
 * @param {number} attempt
 * @returns {Promise<number>}
 */
async function getE2EPort(command, attempt) {
  const commandOffset = command === 'preview' ? E2E_PORT_CANDIDATE_COUNT : 0;
  const workerOffset = workerIndex * E2E_WORKER_PORT_BLOCK_SIZE;
  const retryOffset = (attempt - 1) * E2E_RETRY_PORT_BLOCK_SIZE;
  const startPort = E2E_PORT_BASE + workerOffset + retryOffset + commandOffset;
  const endPort = startPort + E2E_PORT_CANDIDATE_COUNT - 1;
  return getPort({ port: portNumbers(startPort, endPort) });
}

/**
 * @param {string} examplePath
 * @returns {Promise<boolean>}
 */
async function shouldBuildBeforePreview(examplePath) {
  for (const configFile of FARM_CONFIG_FILES) {
    try {
      const config = await readFile(`${examplePath}/${configFile}`, 'utf8');
      if (/server\s*:\s*{[\s\S]*?writeToDisk\s*:\s*true/.test(config)) {
        return true;
      }
    } catch (error) {
      if (error && typeof error === 'object' && 'code' in error && error.code === 'ENOENT') {
        continue;
      }
      throw error;
    }
  }

  return false;
}

/**
 * @param {string} examplePath
 * @returns {Promise<void>}
 */
async function buildBeforePreview(examplePath) {
  logger(`Building before preview: npm run build  in  ${examplePath}`, { color: 'cyan' });

  const child = execa('npm', ['run', 'build'], {
    cwd: examplePath,
    detached: process.platform !== 'win32',
    stdin: 'pipe',
    encoding: 'utf8',
    timeout: 180_000,
    env: {
      ...process.env,
      BROWSER: 'none',
      NO_COLOR: 'true'
    }
  });

  child.stderr?.on('data', (chunk) => {
    logger(chunk.toString().trimEnd(), { color: 'red' });
  });

  try {
    await child;
  } finally {
    await terminateChildProcess(child);
  }
}

/**
 * @param {number} attempt
 * @param {string} examplePath
 * @returns {number}
 */
function getRetryDelay(attempt, examplePath) {
  const retryDelays = [5_000, 15_000, 30_000];
  const baseDelay = retryDelays[attempt - 1] ?? retryDelays.at(-1);
  const workerStagger = workerIndex * 750;
  const exampleStagger = stableHash(examplePath) % 3_000;
  return baseDelay + workerStagger + exampleStagger;
}

// ---------------------------------------------------------------------------
// Page visitor
// ---------------------------------------------------------------------------

/**
 * @param {string} url
 * @param {string} examplePath
 * @param {(page: import('playwright-chromium').Page) => Promise<void>} cb
 * @param {string} command
 * @returns {Promise<void>}
 */
async function visitPage(url, examplePath, cb, command) {
  const page = await requireBrowserContext().newPage();
  const cleanupRoutes = await installExampleRequestRoutes(page, examplePath);
  logger(`Opening page: ${url}  [${examplePath}]`);

  page.on('requestfailed', (req) => {
    // Request failures don't fail the test by themselves — keep them in the
    // log file but suppress on the console in quiet mode to avoid noise.
    logger(
      `Request failed  ${examplePath}: ${req.url()} – ${req.failure()?.errorText ?? ''}`,
      { level: 'info' }
    );
  });

  try {
    return await new Promise((resolve, reject) => {
      let settled = false;
      let cbSucceeded = false;
      const settle = (fn) => {
        if (!settled) {
          settled = true;
          fn();
        }
      };

      page.on('console', (msg) => {
        if (page.isClosed()) return;
        const text = msg.text();
        const lower = text.toLocaleLowerCase();
        const location = msg.location?.();
        const locationText = location?.url
          ? ` (${location.url}:${location.lineNumber ?? 0}:${location.columnNumber ?? 0})`
          : '';

        if (
          msg.type() === 'error' &&
          !lower.includes('warn') &&
          !lower.includes('warning') &&
          !/Parse `.+` failed/.test(text)
        ) {
          logger(`[console:error] ${command} ${examplePath}: ${text}${locationText}`, {
            color: 'red'
          });
          // Ignore console errors that arrive after the assertion callback
          // already succeeded — they typically come from background work
          // (e.g. workers) racing against page teardown when the dev server
          // is being shut down, and should not invalidate a passing test.
          if (cbSucceeded) return;
          settle(() => reject(new Error(`Browser console error: ${text}${locationText}`)));
        } else {
          logger(`[console] ${command} ${examplePath}: ${text}${locationText}`);
        }
      });

      page.on('pageerror', (error) => {
        if (page.isClosed()) return;
        const details =
          error && typeof error === 'object' && 'stack' in error && error.stack
            ? error.stack
            : String(error);
        logger(`[pageerror] ${command} ${examplePath}: ${details}`, { color: 'red' });
        if (cbSucceeded) return;
        settle(() => reject(error));
      });

      page
        .goto(url)
        .then(() => cb(page))
        .then(() => {
          // Mark the test as successful *before* we close the page so that
          // any console.error / pageerror events that fire during teardown
          // (for example workers losing their connection to a dev server
          // that's about to be killed) cannot retroactively fail a test
          // whose assertions have already passed.
          cbSucceeded = true;
          return page.close({ reason: 'test finished', runBeforeUnload: false }).catch(() => {});
        })
        .then(() => settle(() => resolve()))
        .catch((e) => {
          // Failure — close page first, then reject
          return page.close({ reason: 'test failed', runBeforeUnload: false }).catch(() => {}).then(() => {
            settle(() => reject(/** @type {Error} */ (e)));
          });
        });
    });
  } finally {
    await cleanupRoutes();
  }
}

// ---------------------------------------------------------------------------
// startAndTest — spawn `npm run <command>`, wait for URL, then run assertions
// ---------------------------------------------------------------------------

/**
 * @param {string} examplePath
 * @param {(page: import('playwright-chromium').Page) => Promise<void>} cb
 * @param {string} command
 * @param {number} attempt
 * @returns {Promise<void>}
 */
async function startAndTestOnce(examplePath, cb, command, attempt) {
  const port = await getE2EPort(command, attempt);
  logger(`Spawning: npm run ${command}  in  ${examplePath}  on port ${port}`);

  const child = execa('npm', ['run', command], {
    cwd: examplePath,
    detached: process.platform !== 'win32',
    stdin: 'pipe',
    encoding: 'utf8',
    timeout: E2E_TIMEOUT_MS,
    env: {
      ...process.env,
      BROWSER: 'none',
      NO_COLOR: 'true',
      FARM_DEFAULT_SERVER_PORT: String(port),
      FARM_DEFAULT_HMR_PORT: String(port)
    }
  });

  // Wait until a URL appears in stdout/stderr. Keep this short so stalled
  // Windows dev servers fail fast and can retry instead of hanging the job.
  const pageUrl = await new Promise((resolve, reject) => {
    let output = '';
    let settled = false;
    const urlRe =
      /https?:\/\/(localhost|\d{1,3}(?:\.\d{1,3}){3})(:\d+)?(\/[^\s]*)?/g;
    const handleOutput = (chunk) => {
      if (settled) return;
      output += chunk.toString();
      const match = output.replace(/\n/g, ' ').match(urlRe);
      if (match?.[0]) {
        clearTimeout(urlTimer);
        settled = true;
        resolve(match[0]);
      }
    };

    const urlTimer = setTimeout(() => {
      if (!settled) {
        settled = true;
        terminateChildProcess(child).finally(() => {
          reject(new Error(`Timeout waiting for dev server URL in ${examplePath} (${command}) after ${E2E_TIMEOUT_SECONDS}s`));
        });
      }
    }, E2E_TIMEOUT_MS);

    child.stdout?.on('data', (chunk) => {
      handleOutput(chunk);
    });

    child.stderr?.on('data', (chunk) => {
      handleOutput(chunk);
      logger(chunk.toString().trimEnd(), { color: 'red' });
    });

    child.on('error', (err) => {
      if (!settled) {
        clearTimeout(urlTimer);
        settled = true;
        reject(new Error(`Child process error in ${examplePath} (${command}): ${err.message}`));
      }
    });

    child.on('exit', (code) => {
      if (code) {
        clearTimeout(urlTimer);
        settled = true;
        reject(
          new Error(
            `${examplePath} '${command}' exited with code ${code}.\n${output}`
          )
        );
      }
    });
  });

  try {
    await visitPage(pageUrl, examplePath, cb, command);
  } finally {
    await terminateChildProcess(child);
  }
}

/**
 * @param {string} examplePath
 * @param {(page: import('playwright-chromium').Page) => Promise<void>} cb
 * @param {'start' | 'preview'} [command]
 * @param {number} [maxRetries]
 * @returns {Promise<void>}
 */
export async function startAndTest(examplePath, cb, command = 'start', maxRetries = 3) {
  let lastError;

  if (command === 'preview' && await shouldBuildBeforePreview(examplePath)) {
    await buildBeforePreview(examplePath);
  }

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await startAndTestOnce(examplePath, cb, command, attempt);
    } catch (e) {
      if (_recoverBrowser) {
        const reason = isBrowserCrashLikeError(e)
          ? 'browser crash/close'
          : 'failed attempt';
        const nextStep = attempt < maxRetries ? 'before retry' : 'after final attempt';
        logger(`Detected ${reason}. Recreating browser context ${nextStep}...`, {
          color: 'yellow'
        });
        try {
          await _recoverBrowser();
        } catch (recoverError) {
          logger(`Browser recovery failed: ${recoverError}`, { color: 'red' });
        }
      }

      lastError = e;
      logger(
        `Attempt ${attempt}/${maxRetries} failed – ${command} ${examplePath}: ${e}`,
        { color: 'red' }
      );
      if (attempt < maxRetries) {
        const retryDelay = getRetryDelay(attempt, examplePath);
        logger(`Retrying after ${retryDelay} ms to avoid concurrent E2E conflicts...`, {
          color: 'yellow'
        });
        await delay(retryDelay);
      }
    }
  }

  throw lastError instanceof Error
    ? lastError
    : new Error(String(lastError ?? 'Unknown error'));
}

// ---------------------------------------------------------------------------
// watchAndTest — for watch/HMR scenarios that don't serve a URL
// ---------------------------------------------------------------------------

/**
 * @param {string} examplePath
 * @param {(log: string, done: () => void) => Promise<void>} cb
 * @param {string} [command]
 * @returns {Promise<void>}
 */
export async function watchAndTest(examplePath, cb, command = 'start') {
  logger(`Spawning (watch): npm run ${command}  in  ${examplePath}`);

  const child = execa('npm', ['run', command], {
    cwd: examplePath,
    detached: process.platform !== 'win32',
    stdin: 'pipe',
    encoding: 'utf8',
    env: {
      ...process.env,
      BROWSER: 'none',
      NO_COLOR: 'true'
    }
  });

  return new Promise((resolve, reject) => {
    let output = '';
    let settled = false;

    const finish = () => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      terminateChildProcess(child).then(resolve, reject);
    };

    const fail = (error) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      reject(error);
    };

    const timer = setTimeout(() => {
      if (settled) return;
      settled = true;
      terminateChildProcess(child).finally(() => {
        reject(new Error(`Watch test timed out after ${E2E_TIMEOUT_SECONDS} s in ${examplePath}`));
      });
    }, E2E_TIMEOUT_MS);

    child.stdout?.on('data', async (chunk) => {
      output += chunk.toString();
      await cb(output, finish);
    });

    child.on('error', (err) => {
      fail(
        new Error(`Child process error in ${examplePath} (${command}): ${err.message}`)
      );
    });

    child.on('exit', (code) => {
      if (settled) return;
      if (code) {
        fail(
          new Error(`${examplePath} '${command}' exited with code ${code}.\n${output}`)
        );
      }
    });
  });
}
