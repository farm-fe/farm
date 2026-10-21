import path from 'node:path';
import { Worker } from 'node:worker_threads';
import type {
  NotFoundPlan,
  RenderRoute,
  SsgConfig
} from './config.generated.js';
import type { RenderResponse, RenderTask } from './ssg-protocol.js';

interface PoolOptions {
  serverEntry: string;
  snapshotId: string;
  routes: readonly RenderRoute[];
  limits: SsgConfig;
  onPage: (route: RenderRoute, html: string) => Promise<void>;
  notFound?: readonly NotFoundPlan[];
  onNotFound?: (plan: NotFoundPlan, html: string) => Promise<void>;
  signal?: AbortSignal;
}
interface Slot {
  worker: Worker;
  phase: 'starting' | 'idle' | 'rendering' | 'writing';
  timer?: ReturnType<typeof setTimeout>;
  task?: { id: number; job: RenderJob };
}
type RenderJob =
  | { kind: 'page'; route: RenderRoute }
  | { kind: 'notFound'; plan: NotFoundPlan };

/** Execute only Rust-provided page tasks against the compiled SSR artifact. */
export async function renderPages(options: PoolOptions): Promise<void> {
  const { serverEntry, snapshotId, routes, limits, onPage, signal } = options;
  const { concurrency, timeoutMs } = limits;
  if (Boolean(options.notFound) !== (typeof options.onNotFound === 'function'))
    throw new Error(
      '[DOCS_RENDER_PROTOCOL] NotFound plan and output writer must be provided together'
    );
  const jobs: RenderJob[] = routes.map((route) => ({ kind: 'page', route }));
  if (options.notFound)
    for (const plan of options.notFound) jobs.push({ kind: 'notFound', plan });
  if (signal?.aborted)
    throw new Error('[DOCS_RENDER_ABORT] rendering cancelled');
  if (
    !path.isAbsolute(serverEntry) ||
    typeof concurrency !== 'number' ||
    !Number.isInteger(concurrency) ||
    concurrency < 1 ||
    typeof timeoutMs !== 'number' ||
    !Number.isInteger(timeoutMs) ||
    timeoutMs < 1
  )
    throw new Error('[DOCS_RENDER_PROTOCOL] invalid pool inputs');
  if (jobs.length === 0) return;
  const slots: Slot[] = [];
  const writes = new Set<Promise<void>>();
  let next = 0,
    completed = 0,
    settled = false;
  let resolve!: () => void, reject!: (error: Error) => void;
  const completion = new Promise<void>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  function location(slot?: Slot) {
    const job = slot?.task?.job;
    return !job
      ? '<startup>'
      : job.kind === 'notFound'
        ? job.plan.outputPath
        : `${job.route.page.routePath} (${job.route.page.key})`;
  }
  function fail(code: string, message: string, slot?: Slot) {
    if (settled) return;
    settled = true;
    reject(new Error(`[${code}] ${location(slot)}: ${message}`));
  }
  function dispatch(slot: Slot) {
    if (settled || next === jobs.length) return;
    const job = jobs[next],
      id = ++next;
    slot.task = { job, id };
    slot.phase = 'rendering';
    slot.timer = setTimeout(
      () => fail('DOCS_RENDER_TIMEOUT', `page exceeded ${timeoutMs}ms`, slot),
      timeoutMs
    );
    const task: RenderTask =
      job.kind === 'page'
        ? { type: 'render', id, key: job.route.page.key }
        : { type: 'renderNotFound', id, locale: job.plan.locale };
    try {
      slot.worker.postMessage(task);
    } catch (error) {
      fail(
        'DOCS_RENDER_WORKER',
        error instanceof Error ? error.message : String(error),
        slot
      );
    }
  }
  function receive(slot: Slot, value: unknown) {
    if (settled) return;
    if (!value || typeof value !== 'object' || !('type' in value)) {
      fail('DOCS_RENDER_PROTOCOL', 'invalid worker message', slot);
      return;
    }
    const message = value as RenderResponse;
    if (
      message.type === 'fatal' &&
      slot.phase === 'starting' &&
      ['DOCS_RENDER_SNAPSHOT', 'DOCS_RENDER_STARTUP'].includes(message.code) &&
      typeof message.message === 'string'
    ) {
      fail(message.code, message.message, slot);
      return;
    }
    if (
      message.type === 'ready' &&
      slot.phase === 'starting' &&
      message.snapshotId === snapshotId
    ) {
      clearTimeout(slot.timer);
      slot.phase = 'idle';
      dispatch(slot);
      return;
    }
    if (
      (message.type !== 'result' && message.type !== 'error') ||
      slot.phase !== 'rendering' ||
      !slot.task ||
      message.id !== slot.task.id
    ) {
      fail(
        'DOCS_RENDER_PROTOCOL',
        'worker response has no matching active task',
        slot
      );
      return;
    }
    clearTimeout(slot.timer);
    if (message.type === 'error') {
      fail(
        'DOCS_RENDER_PAGE',
        typeof message.message === 'string'
          ? message.message
          : 'invalid render error',
        slot
      );
      return;
    }
    if (typeof message.html !== 'string') {
      fail('DOCS_RENDER_PROTOCOL', 'invalid HTML payload', slot);
      return;
    }
    slot.phase = 'writing';
    const job = slot.task.job;
    const writing = Promise.resolve()
      .then(() => {
        if (settled) return;
        if (job.kind === 'page') return onPage(job.route, message.html);
        const writer = options.onNotFound;
        if (!writer)
          throw new Error(
            '[DOCS_RENDER_PROTOCOL] NotFound output writer missing'
          );
        return writer(job.plan, message.html);
      })
      .then(() => {
        if (settled) return;
        completed++;
        slot.phase = 'idle';
        slot.task = undefined;
        if (completed === jobs.length) {
          settled = true;
          resolve();
        } else dispatch(slot);
      })
      .catch((error) =>
        fail(
          'DOCS_RENDER_OUTPUT',
          error instanceof Error ? error.message : String(error),
          slot
        )
      )
      .finally(() => {
        writes.delete(writing);
      });
    writes.add(writing);
  }
  const abort = () => fail('DOCS_RENDER_ABORT', 'rendering cancelled');
  signal?.addEventListener('abort', abort, { once: true });
  let failed = false,
    failure: unknown;
  try {
    if (signal?.aborted) abort();
    for (
      let index = 0;
      !settled && index < Math.min(concurrency, jobs.length);
      index++
    ) {
      const worker = new Worker(new URL('./ssg-worker.js', import.meta.url), {
        workerData: { serverEntry, snapshotId },
        execArgv: []
      });
      const slot: Slot = { worker, phase: 'starting' };
      slots.push(slot);
      slot.timer = setTimeout(
        () =>
          fail(
            'DOCS_RENDER_STARTUP_TIMEOUT',
            'server initialization exceeded its deadline',
            slot
          ),
        Math.max(1000, timeoutMs)
      );
      worker.on('message', (value) => receive(slot, value));
      worker.on('error', (error) =>
        fail('DOCS_RENDER_WORKER', error.message, slot)
      );
      worker.on('exit', (code) =>
        fail('DOCS_RENDER_WORKER', `worker exited with code ${code}`, slot)
      );
    }
    await completion;
  } catch (error) {
    failed = true;
    failure = error;
  } finally {
    signal?.removeEventListener('abort', abort);
    settled = true;
    for (const slot of slots) clearTimeout(slot.timer);
    const termination = await Promise.allSettled(
      slots.map((slot) => slot.worker.terminate())
    );
    await Promise.allSettled([...writes]);
    const cleanup = termination.flatMap((item) =>
      item.status === 'rejected' ? [item.reason] : []
    );
    if (cleanup.length) {
      if (failed) cleanup.unshift(failure);
      failed = true;
      failure = new AggregateError(
        cleanup,
        '[DOCS_RENDER_CLEANUP] worker termination failed'
      );
    }
  }
  if (failed) throw failure;
}
