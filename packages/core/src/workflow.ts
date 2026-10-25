import { createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import type { commandType, ResolvedUserConfig } from './config/types.js';

export interface WorkflowContext {
  command: commandType;
  /** The resolved config with workflow removed to prevent recursive dispatch. */
  config: ResolvedUserConfig;
  options?: Record<string, unknown>;
  /** Runtime cancellation only; never serialized into compiler configuration. */
  signal?: AbortSignal;
}

export interface WorkflowExecutionOptions {
  signal?: AbortSignal;
}

export function throwIfWorkflowCancelled(signal?: AbortSignal): void {
  if (signal?.aborted)
    throw Object.assign(
      new Error('[FARM_WORKFLOW_CANCELLED] workflow cancelled'),
      { cause: signal.reason }
    );
}

/** Dispatch trusted project configuration, never a compilation hook. */
export async function runConfiguredWorkflow(
  config: ResolvedUserConfig,
  command: commandType,
  execution: WorkflowExecutionOptions = {}
): Promise<boolean> {
  const workflow = config.workflow;
  if (!workflow) return false;
  const controller = command === 'build' ? new AbortController() : undefined;
  const signal = controller?.signal ?? execution.signal;
  const forwardAbort = () => controller?.abort(execution.signal?.reason);
  let connected = false;
  if (controller && execution.signal) {
    if (execution.signal.aborted) forwardAbort();
    else {
      execution.signal.addEventListener('abort', forwardAbort, { once: true });
      connected = true;
    }
  }
  const interrupt = () =>
    controller?.abort(new Error('[FARM_WORKFLOW_CANCELLED] SIGINT'));
  const terminate = () =>
    controller?.abort(new Error('[FARM_WORKFLOW_CANCELLED] SIGTERM'));
  if (controller) {
    process.on('SIGINT', interrupt);
    process.on('SIGTERM', terminate);
  }
  try {
    throwIfWorkflowCancelled(signal);
    const root = path.resolve(config.root ?? process.cwd());
    const specifier = workflow.module;
    let modulePath: string;
    if (specifier.startsWith('file:')) {
      modulePath = fileURLToPath(specifier);
    } else if (path.isAbsolute(specifier) || specifier.startsWith('.')) {
      modulePath = path.resolve(root, specifier);
    } else {
      modulePath = createRequire(path.join(root, 'package.json')).resolve(
        specifier
      );
    }
    const runner = await import(pathToFileURL(modulePath).href);
    throwIfWorkflowCancelled(signal);
    if (typeof runner.run !== 'function') {
      throw new Error(
        `[FARM_WORKFLOW_EXPORT] ${specifier} must export a run function`
      );
    }
    const { workflow: _workflow, ...childConfig } = config;
    await runner.run({
      command,
      config: childConfig,
      options: workflow.options,
      signal
    } satisfies WorkflowContext);
    return true;
  } finally {
    if (controller) {
      process.off('SIGINT', interrupt);
      process.off('SIGTERM', terminate);
    }
    if (connected) execution.signal?.removeEventListener('abort', forwardAbort);
  }
}
