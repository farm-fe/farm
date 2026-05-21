import { appendFileSync, mkdirSync, existsSync } from 'node:fs';
import { createWriteStream } from 'node:fs';
import * as fs from 'node:fs/promises';
import { isAbsolute, join, dirname } from 'node:path';

// ---------------------------------------------------------------------------
// File-based logging
// ---------------------------------------------------------------------------

/** @type {Map<string, import('node:fs').WriteStream>} */
const _logStreams = new Map();

/** @type {string | null} */
let _defaultFilePath = null;

/**
 * Set the default log file path. All logger() calls will also append to this file.
 * @param {string} filePath
 */
export function setLogFile(filePath) {
  _defaultFilePath = filePath;
}

/**
 * Creates a write stream for a given log file (creates parent dirs if needed).
 * Returns the stream, reusing an existing one if already open.
 * @param {string} filePath
 * @returns {import('node:fs').WriteStream}
 */
function getOrCreateStream(filePath) {
  let s = _logStreams.get(filePath);
  if (!s) {
    const dir = dirname(filePath);
    if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
    s = createWriteStream(filePath, { flags: 'a' });
    _logStreams.set(filePath, s);
  }
  return s;
}

/**
 * Close all open log file streams synchronously to ensure buffers are flushed.
 */
export function closeLogFiles() {
  for (const [path, stream] of _logStreams) {
    try {
      // WriteStream is synchronous when created without 'a' deferred write — use end() then
      // ensure it is properly closed before process exit by using closeSync on the underlying fd
      stream.end();
    } catch {}
  }
  _logStreams.clear();
  _defaultFilePath = null;
}

// ---------------------------------------------------------------------------
// Console + file logger
// ---------------------------------------------------------------------------

/**
 * @param {*} msg
 * @param {{ title?: string, color?: string, file?: string }} [opts]
 */
export function logger(msg, { title = 'FARM INFO', color = 'green', file } = {}) {
  const COLOR_CODE = [
    'black',
    'red',
    'green',
    'yellow',
    'blue',
    'magenta',
    'cyan',
    'white'
  ].indexOf(color);

  // Format for console (with ANSI)
  let consoleLine;
  if (COLOR_CODE >= 0) {
    const TITLE_STR = title ? `\x1b[4${COLOR_CODE};30m ${title} \x1b[0m ` : '';
    consoleLine = `${TITLE_STR}\x1b[3${COLOR_CODE}m${msg}\x1b[;0m`;
  } else {
    consoleLine = title ? `${title} ${msg}` : String(msg);
  }
  console.log(consoleLine);

  // Format for file (no ANSI, with timestamp)
  const now = new Date().toISOString();
  const plain = title ? `[${now}] [${title}] ${msg}` : `[${now}] ${msg}`;

  // Write to explicit file, default file, or both
  if (file) {
    getOrCreateStream(file).write(plain + '\n');
  }
  if (_defaultFilePath && file !== _defaultFilePath) {
    getOrCreateStream(_defaultFilePath).write(plain + '\n');
  }
}

/**
 * @template T
 * @param {boolean} [silent]
 * @returns {{ resolve: (result: T) => void, reject: (reason: any) => void, promise: Promise<T> }}
 */
export function createDeferred(silent) {
  /** @type {any} */
  const deferred = {};

  deferred.promise = new Promise((resolve, reject) => {
    deferred.resolve = resolve;
    deferred.reject = reject;
  });

  if (silent) {
    deferred.promise.catch(() => {});
  }

  return deferred;
}

/**
 * @template F
 * @param {number} maxConcurrent
 * @param {F} fn
 * @returns {F}
 */
export function concurrentify(maxConcurrent, fn) {
  const queue = [];

  let concurrent = 0;

  function next() {
    concurrent -= 1;
    let task = queue.shift();
    if (task) {
      const { ctx, deferred, args } = task;
      try {
        newFn.apply(ctx, args).then(deferred.resolve, deferred.reject);
      } catch (e) {
        deferred.reject(e);
      }
    }
  }

  function newFn() {
    const ctx = this;
    const args = arguments;

    if (concurrent >= maxConcurrent) {
      const deferred = createDeferred();
      queue.push({ deferred, ctx, args });
      return deferred.promise;
    }

    concurrent += 1;

    return fn.apply(ctx, args).finally(next);
  }

  return /** @type {F} */ (newFn);
}

/**
 * @template {readonly unknown[]} Arr
 * @template F
 * @param {Arr} arr
 * @param {number} maxConcurrent
 * @param {F} cb
 * @returns {ReturnType<F>[]}
 */
export function concurrentMap(arr, maxConcurrent, cb) {
  return arr.map(concurrentify(maxConcurrent, cb));
}

/**
 * @param {string} _filename
 * @param {string | RegExp} matched
 * @param {string} to
 * @returns {Promise<undefined | (() => Promise<void>)>}
 */
export async function editFile(_filename, matched, to) {
  const filename = isAbsolute(_filename)
    ? _filename
    : join(process.cwd(), _filename);
  const content = await fs.readFile(filename, 'utf-8');

  let newContent = content.replaceAll(matched, to);

  if (content.length !== newContent.length || content !== newContent) {
    await fs.writeFile(filename, newContent);

    return async () => {
      await fs.writeFile(filename, content);
    };
  }
}
