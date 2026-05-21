import { execa } from 'execa';
import getPort from 'get-port';
import { logger } from './utils.mjs';

// ---------------------------------------------------------------------------
// Browser singleton
// ---------------------------------------------------------------------------

let _browser = null;
let _context = null;
let _recoverBrowser = null;

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
  logger(`Opening page: ${url}  [${examplePath}]`);

  page.on('requestfailed', (req) => {
    // Request failures don't fail the test by themselves — keep them in the
    // log file but suppress on the console in quiet mode to avoid noise.
    logger(
      `Request failed  ${examplePath}: ${req.url()} – ${req.failure()?.errorText ?? ''}`,
      { level: 'info' }
    );
  });

  return new Promise((resolve, reject) => {
    let settled = false;
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
      settle(() => reject(error));
    });

    page
      .goto(url)
      .then(() => cb(page))
      .then(() => {
        // Success — close page first, then resolve
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
}

// ---------------------------------------------------------------------------
// startAndTest — spawn `npm run <command>`, wait for URL, then run assertions
// ---------------------------------------------------------------------------

/**
 * @param {string} examplePath
 * @param {(page: import('playwright-chromium').Page) => Promise<void>} cb
 * @param {string} command
 * @returns {Promise<void>}
 */
async function startAndTestOnce(examplePath, cb, command) {
  const port = await getPort();
  logger(`Spawning: npm run ${command}  in  ${examplePath}`);

  const child = execa('npm', ['run', command], {
    cwd: examplePath,
    stdin: 'pipe',
    encoding: 'utf8',
    timeout: 120_000, // 2-minute timeout for dev server startup
    env: {
      ...process.env,
      BROWSER: 'none',
      NO_COLOR: 'true',
      FARM_DEFAULT_SERVER_PORT: String(port),
      FARM_DEFAULT_HMR_PORT: String(port)
    }
  });

  // Wait until a URL appears in stdout (60s timeout)
  const pageUrl = await new Promise((resolve, reject) => {
    let output = '';
    let settled = false;
    const urlRe =
      /https?:\/\/(localhost|\d{1,3}(?:\.\d{1,3}){3})(:\d+)?(\/[^\s]*)?/g;

    const urlTimer = setTimeout(() => {
      if (!settled) {
        settled = true;
        if (!child.killed) child.kill('SIGTERM');
        reject(new Error(`Timeout waiting for dev server URL in ${examplePath} (${command}) after 60s`));
      }
    }, 60_000);

    child.stdout?.on('data', (chunk) => {
      if (settled) return;
      output += chunk.toString();
      const match = output.replace(/\n/g, ' ').match(urlRe);
      if (match?.[0]) {
        clearTimeout(urlTimer);
        settled = true;
        resolve(match[0]);
      }
    });

    child.stderr?.on('data', (chunk) => {
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
    if (!child.killed) child.kill('SIGTERM');
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

  const delays = [2_000, 5_000, 10_000];

  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await startAndTestOnce(examplePath, cb, command);
    } catch (e) {
      if (_recoverBrowser && isBrowserCrashLikeError(e)) {
        logger('Detected browser crash/close. Recreating browser before retry...', {
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
        const delay = delays[attempt - 1] || 10_000;
        await new Promise((r) => setTimeout(r, delay));
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

    const timer = setTimeout(() => {
      if (!child.killed) child.kill('SIGTERM');
      reject(new Error(`Watch test timed out after 60 s in ${examplePath}`));
    }, 60_000);

    child.stdout?.on('data', async (chunk) => {
      output += chunk.toString();
      await cb(output, () => {
        clearTimeout(timer);
        if (!child.killed) child.kill('SIGTERM');
        resolve();
      });
    });

    child.on('error', (err) => {
      clearTimeout(timer);
      reject(
        new Error(`Child process error in ${examplePath} (${command}): ${err.message}`)
      );
    });

    child.on('exit', (code) => {
      if (code) {
        clearTimeout(timer);
        reject(
          new Error(`${examplePath} '${command}' exited with code ${code}.\n${output}`)
        );
      }
    });
  });
}
