/**
 * Farm process management + Playwright page utilities.
 *
 * Replaces the vitest-specific logic in the old vitestSetup.ts.
 * No dependency on vitest at all — uses a browser singleton set by the runner.
 */
import { execa } from 'execa';
import type { Browser, BrowserContext, Page } from 'playwright-chromium';
import getPort from 'get-port';
import { logger } from './utils.ts';

// ---------------------------------------------------------------------------
// Browser singleton
// ---------------------------------------------------------------------------

let _browser: Browser | null = null;
let _context: BrowserContext | null = null;
let _recoverBrowser: (() => Promise<void>) | null = null;

/** Called once by the test runner before any spec runs. */
export function initBrowser(browser: Browser): void {
  _browser = browser;
}

/** Called by the runner when rotating contexts between example groups. */
export function initBrowserContext(context: BrowserContext): void {
  _context = context;
}

/** Optional hook used by the outer runner to recreate browser on crash. */
export function setBrowserRecoveryHandler(handler: (() => Promise<void>) | null): void {
  _recoverBrowser = handler;
}

function isBrowserCrashLikeError(error: unknown): boolean {
  const msg = error instanceof Error ? error.message : String(error ?? '');
  return (
    msg.includes('Target page, context or browser has been closed') ||
    msg.includes('Target crashed') ||
    msg.includes('Page crashed')
  );
}

function requireBrowser(): Browser {
  if (!_browser) {
    throw new Error('Browser not initialised. Call initBrowser() before running specs.');
  }
  return _browser;
}

function requireBrowserContext(): BrowserContext {
  if (_context) {
    return _context;
  }

  // Keep the error actionable while preserving backwards compatibility with old setup.
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

async function visitPage(
  url: string,
  examplePath: string,
  cb: (page: Page) => Promise<void>,
  command: string
): Promise<void> {
  const page = await requireBrowserContext().newPage();
  logger(`Opening page: ${url}  [${examplePath}]`);

  page.on('requestfailed', (req) => {
    logger(
      `Request failed  ${examplePath}: ${req.url()} – ${req.failure()?.errorText ?? ''}`,
      { color: 'red' }
    );
  });

  return new Promise<void>((resolve, reject) => {
    let settled = false;
    const settle = (fn: () => void): void => {
      if (!settled) {
        settled = true;
        fn();
      }
    };

    page.on('console', (msg) => {
      if (page.isClosed()) return;
      const text = msg.text();
      const lower = text.toLocaleLowerCase();

      if (
        msg.type() === 'error' &&
        !lower.includes('warn') &&
        !lower.includes('warning') &&
        !/Parse `.+` failed/.test(text)
      ) {
        logger(`[console:error] ${command} ${examplePath}: ${text}`, { color: 'red' });
        settle(() => reject(new Error(`Browser console error: ${text}`)));
      } else {
        logger(`[console] ${command} ${examplePath}: ${text}`);
      }
    });

    page.on('pageerror', (error) => {
      if (page.isClosed()) return;
      logger(`[pageerror] ${command} ${examplePath}: ${error}`, { color: 'red' });
      settle(() => reject(error));
    });

    page
      .goto(url)
      .then(() => {
        cb(page)
          .then(() => settle(() => resolve()))
          .catch((e: unknown) => settle(() => reject(e as Error)))
          .finally(() => {
            page.close({ reason: 'test finished', runBeforeUnload: false }).catch(() => {});
          });
      })
      .catch((e: unknown) => {
        page.close().catch(() => {});
        settle(() => reject(e as Error));
      });
  });
}

// ---------------------------------------------------------------------------
// startAndTest — spawn `npm run <command>`, wait for URL, then run assertions
// ---------------------------------------------------------------------------

async function startAndTestOnce(
  examplePath: string,
  cb: (page: Page) => Promise<void>,
  command: string
): Promise<void> {
  const port = await getPort();
  logger(`Spawning: npm run ${command}  in  ${examplePath}`);

  const child = execa('npm', ['run', command], {
    cwd: examplePath,
    stdin: 'pipe',
    encoding: 'utf8',
    env: {
      ...process.env,
      BROWSER: 'none',
      NO_COLOR: 'true',
      FARM_DEFAULT_SERVER_PORT: String(port),
      FARM_DEFAULT_HMR_PORT: String(port)
    }
  });

  // Wait until a URL appears in stdout
  const pageUrl = await new Promise<string>((resolve, reject) => {
    let output = '';
    const urlRe =
      /https?:\/\/(localhost|\d{1,3}(?:\.\d{1,3}){3})(:\d+)?(\/[^\s]*)?/g;

    child.stdout?.on('data', (chunk: Buffer) => {
      output += chunk.toString();
      const match = output.replace(/\n/g, ' ').match(urlRe);
      if (match?.[0]) resolve(match[0]);
    });

    child.stderr?.on('data', (chunk: Buffer) => {
      logger(chunk.toString().trimEnd(), { color: 'red' });
    });

    child.on('error', (err: Error) => {
      reject(new Error(`Child process error in ${examplePath} (${command}): ${err.message}`));
    });

    child.on('exit', (code: number | null) => {
      if (code) {
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
    // SIGTERM ensures the spawned dev/preview process is really terminated.
    if (!child.killed) child.kill('SIGTERM');
  }
}

/**
 * Start the Farm dev/preview server for `examplePath`, open a Playwright
 * page at the emitted URL, and run `cb` against that page.
 *
 * Retries up to `maxRetries` times with a 10 s delay between attempts.
 */
export async function startAndTest(
  examplePath: string,
  cb: (page: Page) => Promise<void>,
  command: 'start' | 'preview' = 'start',
  maxRetries = 3
): Promise<void> {
  let lastError: unknown;

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
        await new Promise((r) => setTimeout(r, 10_000));
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
 * Spawn `npm run <command>` and stream stdout to `cb`.
 * `cb` receives the accumulated output so far and a `done()` callback to
 * signal completion.  Times out after 60 s.
 */
export async function watchAndTest(
  examplePath: string,
  cb: (log: string, done: () => void) => Promise<void>,
  command = 'start'
): Promise<void> {
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

  return new Promise<void>((resolve, reject) => {
    let output = '';

    const timer = setTimeout(() => {
      if (!child.killed) child.kill('SIGTERM');
      reject(new Error(`Watch test timed out after 60 s in ${examplePath}`));
    }, 60_000);

    child.stdout?.on('data', async (chunk: Buffer) => {
      output += chunk.toString();
      await cb(output, () => {
        clearTimeout(timer);
        if (!child.killed) child.kill('SIGTERM');
        resolve();
      });
    });

    child.on('error', (err: Error) => {
      clearTimeout(timer);
      reject(
        new Error(`Child process error in ${examplePath} (${command}): ${err.message}`)
      );
    });

    child.on('exit', (code: number | null) => {
      if (code) {
        clearTimeout(timer);
        reject(
          new Error(`${examplePath} '${command}' exited with code ${code}.\n${output}`)
        );
      }
    });
  });
}

export type { Page };
