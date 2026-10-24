import { createRequire } from 'node:module';
import { parentPort, workerData } from 'node:worker_threads';
import type { RenderResponse, RenderTask } from './ssg-protocol.js';

const port = parentPort;
if (!port)
  throw new Error('[DOCS_RENDER_STARTUP] worker requires a parent port');
const send = (message: RenderResponse) => port.postMessage(message);
try {
  // serverEntry is the workflow's compiled private artifact, never a content source.
  const renderer = createRequire(import.meta.url)(workerData.serverEntry);
  if (renderer.snapshotId !== workerData.snapshotId) {
    send({
      type: 'fatal',
      code: 'DOCS_RENDER_SNAPSHOT',
      message: 'server snapshot mismatch'
    });
  } else if (typeof renderer.render !== 'function') {
    send({
      type: 'fatal',
      code: 'DOCS_RENDER_STARTUP',
      message: 'server render export missing'
    });
  } else {
    port.on('message', async (task: RenderTask) => {
      if (
        !task ||
        !Number.isSafeInteger(task.id) ||
        (task.type === 'renderNotFound' &&
          (typeof task.locale !== 'string' || !task.locale)) ||
        (task.type !== 'renderNotFound' &&
          (task.type !== 'render' || typeof task.key !== 'string'))
      ) {
        send({
          type: 'fatal',
          code: 'DOCS_RENDER_STARTUP',
          message: 'invalid render task'
        });
        return;
      }
      try {
        if (
          task.type === 'renderNotFound' &&
          typeof renderer.renderNotFound !== 'function'
        )
          throw new Error('server renderNotFound export missing');
        const html: unknown =
          task.type === 'renderNotFound'
            ? await renderer.renderNotFound(task.locale)
            : await renderer.render(task.key);
        if (typeof html !== 'string')
          throw new Error('render must return an HTML string');
        send({ type: 'result', id: task.id, html });
      } catch (error) {
        send({
          type: 'error',
          id: task.id,
          message: error instanceof Error ? error.message : String(error)
        });
      }
    });
    send({ type: 'ready', snapshotId: renderer.snapshotId });
  }
} catch (error) {
  send({
    type: 'fatal',
    code: 'DOCS_RENDER_STARTUP',
    message: error instanceof Error ? error.message : String(error)
  });
}
