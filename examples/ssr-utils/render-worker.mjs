import { parentPort, workerData } from 'node:worker_threads';

try {
  const { default: render } = await import(workerData.entry);
  const html = await render(workerData.url);
  if (typeof html !== 'string') throw new TypeError('SSR render must return HTML');
  parentPort.postMessage({ html });
} catch (error) {
  parentPort.postMessage({ error: error instanceof Error ? error : new Error(String(error)) });
}
