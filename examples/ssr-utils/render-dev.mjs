import { pathToFileURL } from 'node:url';
import { Worker } from 'node:worker_threads';

// Development only: each request gets a fresh module graph, including lazy chunks.
export async function renderDev(entry, url, {
  timeoutMs = 15000,
  workerUrl = new URL('./render-worker.mjs', import.meta.url)
} = {}) {
  if (!Number.isFinite(timeoutMs) || timeoutMs <= 0) {
    throw new TypeError('timeoutMs must be positive and finite');
  }
  const worker = new Worker(workerUrl, {
    workerData: { entry: pathToFileURL(entry).href, url },
    execArgv: []
  });
  let timer;
  try {
    return await new Promise((resolve, reject) => {
      timer = setTimeout(() => reject(new Error('Development SSR render timed out')), timeoutMs);
      worker.once('message', result => {
        if (result.error) reject(result.error);
        else if (typeof result.html === 'string') resolve(result.html);
        else reject(new Error('Invalid SSR render result'));
      });
      worker.once('error', reject);
      worker.once('exit', code => reject(new Error(`SSR worker exited ${code} without a render result`)));
    });
  } finally {
    clearTimeout(timer);
    await worker.terminate();
  }
}
