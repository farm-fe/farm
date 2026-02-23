import { cleanUrl } from '../utils/url.js';
import {
  farmSsrDynamicImportKey,
  farmSsrExportAllKey,
  farmSsrExportNameKey,
  farmSsrImportKey,
  farmSsrImportMetaKey,
  farmSsrModuleExportsKey
} from './constants.js';
import type { EvaluatedModuleNode } from './evaluatedModules.js';
import type {
  ExternalFetchResult,
  ExternalModuleResolveResult,
  ExternalModuleResolver,
  FarmRunnerContext,
  ModuleEvaluator,
  ModuleRunnerEvaluatorType
} from './types.js';

// eslint-disable-next-line @typescript-eslint/no-implied-eval
const AsyncFunction = async function () {}.constructor as FunctionConstructor;

type NodeRequireFunction = (id: string) => unknown;
type NodeCreateRequire = (url: string) => NodeRequireFunction;
type NodeFileUrlToPath = (url: string) => string;
type ImportErrorLike = {
  code?: unknown;
  message?: unknown;
  cause?: unknown;
};

const IMPORT_FALLBACK_ERROR_CODES = new Set([
  'ERR_MODULE_NOT_FOUND',
  'ERR_UNSUPPORTED_ESM_URL_SCHEME',
  'ERR_INVALID_URL',
  'ERR_INVALID_MODULE_SPECIFIER',
  'MODULE_NOT_FOUND'
]);

const IMPORT_FALLBACK_TYPED_ERROR_MESSAGE_PATTERNS: RegExp[] = [
  /failed to resolve module specifier/i,
  /only urls with a scheme in/i,
  /unsupported.+scheme/i,
  /invalid url/i,
  /cannot load.+https/i
];

const IMPORT_FALLBACK_STRICT_ERROR_MESSAGE_PATTERNS: RegExp[] = [
  /^failed to load url\b/i
];

function getProcessLike(): Record<string, unknown> | undefined {
  const processLike = Reflect.get(globalThis as object, 'process');

  if (!processLike || typeof processLike !== 'object') {
    return undefined;
  }

  return processLike as Record<string, unknown>;
}

function getNodeBuiltinModule(
  moduleName: 'node:module' | 'module' | 'node:url' | 'url'
): Record<string, unknown> | undefined {
  const processLike = getProcessLike();

  if (!processLike) {
    return undefined;
  }

  const getBuiltinModule = Reflect.get(processLike, 'getBuiltinModule');

  if (typeof getBuiltinModule !== 'function') {
    return undefined;
  }

  try {
    return Reflect.apply(
      getBuiltinModule as (...args: unknown[]) => unknown,
      processLike,
      [moduleName]
    ) as Record<string, unknown> | undefined;
  } catch {
    return undefined;
  }
}

function resolveNodeCreateRequire(): NodeCreateRequire | undefined {
  for (const name of ['node:module', 'module'] as const) {
    const moduleBuiltin = getNodeBuiltinModule(name);
    const createRequire =
      moduleBuiltin && Reflect.get(moduleBuiltin, 'createRequire');

    if (typeof createRequire === 'function') {
      return createRequire as NodeCreateRequire;
    }
  }

  return undefined;
}

function resolveNodeFileURLToPath(): NodeFileUrlToPath | undefined {
  for (const name of ['node:url', 'url'] as const) {
    const urlBuiltin = getNodeBuiltinModule(name);
    const fileURLToPath =
      urlBuiltin && Reflect.get(urlBuiltin, 'fileURLToPath');

    if (typeof fileURLToPath === 'function') {
      return fileURLToPath as NodeFileUrlToPath;
    }
  }

  return undefined;
}

function fileUrlToPathWithFallback(url: string): string {
  const native = resolveNodeFileURLToPath();

  if (native) {
    return native(url);
  }

  const parsed = new URL(url);
  let pathname = decodeURIComponent(parsed.pathname);

  if (/^\/[a-zA-Z]:\//.test(pathname)) {
    pathname = pathname.slice(1);
  }

  return pathname;
}

function encodeBase64Utf8(value: string): string | undefined {
  const globalBuffer = Reflect.get(globalThis as object, 'Buffer') as
    | {
        from?: (
          input: string,
          encoding?: string
        ) => { toString: (encoding: string) => string };
      }
    | undefined;

  if (globalBuffer && typeof globalBuffer.from === 'function') {
    return globalBuffer.from(value, 'utf-8').toString('base64');
  }

  if (typeof TextEncoder !== 'function' || typeof btoa !== 'function') {
    return undefined;
  }

  const bytes = new TextEncoder().encode(value);
  let binary = '';

  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }

  return btoa(binary);
}

function toInlineSourceMapComment(sourceMap: string): string | undefined {
  const encoded = encodeBase64Utf8(sourceMap);

  if (!encoded) {
    return undefined;
  }

  return `//# sourceMappingURL=data:application/json;base64,${encoded}`;
}

function getImportErrorCode(error: unknown): string | undefined {
  if (!error || typeof error !== 'object') {
    return undefined;
  }

  const code = (error as ImportErrorLike).code;
  return typeof code === 'string' ? code : undefined;
}

function getImportErrorMessage(error: unknown): string | undefined {
  if (error instanceof Error) {
    return error.message;
  }

  if (!error || typeof error !== 'object') {
    return undefined;
  }

  const message = (error as ImportErrorLike).message;
  return typeof message === 'string' ? message : undefined;
}

function getImportErrorName(error: unknown): string | undefined {
  if (error instanceof Error) {
    return error.name;
  }

  if (!error || typeof error !== 'object') {
    return undefined;
  }

  const name = Reflect.get(error as object, 'name');
  return typeof name === 'string' ? name : undefined;
}

function shouldTryImportFallback(error: unknown): boolean {
  const queue: unknown[] = [error];
  const seen = new Set<unknown>();

  while (queue.length) {
    const current = queue.shift();

    if (!current || seen.has(current)) {
      continue;
    }

    seen.add(current);

    const code = getImportErrorCode(current);
    if (code && IMPORT_FALLBACK_ERROR_CODES.has(code)) {
      return true;
    }

    const message = getImportErrorMessage(current);
    const name = getImportErrorName(current);
    if (message) {
      if (
        (name === 'TypeError' || name === 'URIError') &&
        IMPORT_FALLBACK_TYPED_ERROR_MESSAGE_PATTERNS.some((pattern) =>
          pattern.test(message)
        )
      ) {
        return true;
      }

      if (
        (!name || name === 'Error') &&
        IMPORT_FALLBACK_STRICT_ERROR_MESSAGE_PATTERNS.some((pattern) =>
          pattern.test(message)
        )
      ) {
        return true;
      }
    }

    if (typeof current === 'object') {
      const cause = (current as ImportErrorLike).cause;
      if (cause) {
        queue.push(cause);
      }
    }
  }

  return false;
}

function normalizeResolveResult(result: unknown): ExternalModuleResolveResult {
  if (result == null) {
    return { resolved: false };
  }

  if (typeof result !== 'object') {
    throw new Error(
      `[farm module runner] resolveExternalModule must return an object with boolean "resolved", but got ${typeof result}.`
    );
  }

  const resolved = Reflect.get(result as object, 'resolved');

  if (typeof resolved !== 'boolean') {
    throw new Error(
      '[farm module runner] resolveExternalModule must return an object with boolean "resolved".'
    );
  }

  if (resolved) {
    return {
      resolved: true,
      module: Reflect.get(result as object, 'module')
    };
  }

  return { resolved: false };
}

async function runInlinedModuleWithAsyncFunction(
  context: FarmRunnerContext,
  code: string,
  module: Readonly<EvaluatedModuleNode>
): Promise<void> {
  const sourceComments: string[] = [];
  const sourceId =
    module.meta && 'file' in module.meta && module.meta.file
      ? module.meta.file
      : module.id;

  if (sourceId) {
    sourceComments.push(`//# sourceURL=${sourceId}`);
  }

  if (module.meta && 'map' in module.meta && module.meta.map) {
    const sourceMapComment = toInlineSourceMapComment(module.meta.map);
    if (sourceMapComment) {
      sourceComments.push(sourceMapComment);
    }
  }

  const instrumentedCode = ['"use strict";', code, sourceComments.join('\n')]
    .filter(Boolean)
    .join('\n');

  const initModule = new AsyncFunction(
    farmSsrModuleExportsKey,
    farmSsrImportMetaKey,
    farmSsrImportKey,
    farmSsrDynamicImportKey,
    farmSsrExportAllKey,
    farmSsrExportNameKey,
    instrumentedCode
  );

  await initModule(
    context[farmSsrModuleExportsKey],
    context[farmSsrImportMetaKey],
    context[farmSsrImportKey],
    context[farmSsrDynamicImportKey],
    context[farmSsrExportAllKey],
    context[farmSsrExportNameKey]
  );

  Object.seal(context[farmSsrModuleExportsKey]);
}

async function importModuleWithOptions(
  file: string,
  options?: unknown
): Promise<unknown> {
  if (options === undefined) {
    return import(/* @vite-ignore */ file);
  }

  return import(/* @vite-ignore */ file, options as Record<string, unknown>);
}

class BaseAsyncFunctionEvaluator implements ModuleEvaluator {
  readonly startOffset = 2;
  private readonly nodeCommonJsLoader: NodeRequireFunction | undefined;

  constructor(
    private readonly runtimeName: ModuleRunnerEvaluatorType,
    private readonly allowCommonJsRequire: boolean,
    private readonly resolveExternalModule?: ExternalModuleResolver
  ) {
    this.nodeCommonJsLoader = allowCommonJsRequire
      ? this.createNodeCommonJsLoader()
      : undefined;
  }

  async runInlinedModule(
    context: FarmRunnerContext,
    code: string,
    module: Readonly<EvaluatedModuleNode>
  ): Promise<void> {
    await runInlinedModuleWithAsyncFunction(context, code, module);
  }

  async runExternalModule(
    file: string,
    type: ExternalFetchResult['type'] = 'module',
    options?: unknown
  ): Promise<unknown> {
    if (type === 'builtin' && this.runtimeName !== 'node') {
      const unsupportedBuiltinError = new Error(
        `[farm module runner] builtin external module is not supported in ${this.runtimeName} evaluator.`
      );
      const resolved = await this.resolveUnsupportedExternal(file, type, {
        reason: 'unsupported-external',
        error: unsupportedBuiltinError,
        importOptions: options
      });
      if (resolved.resolved) {
        return resolved.module;
      }

      throw unsupportedBuiltinError;
    }

    if (type === 'commonjs') {
      if (!this.allowCommonJsRequire) {
        const unsupportedCommonJsError = new Error(
          `[farm module runner] commonjs external module is not supported in ${this.runtimeName} evaluator.`
        );
        const resolved = await this.resolveUnsupportedExternal(file, type, {
          reason: 'unsupported-external',
          error: unsupportedCommonJsError,
          importOptions: options
        });
        if (resolved.resolved) {
          return resolved.module;
        }

        throw unsupportedCommonJsError;
      }

      if (!this.nodeCommonJsLoader) {
        const missingNodeLoaderError = new Error(
          '[farm module runner] commonjs external module requires Node module loader support, but current host does not provide it.'
        );
        const resolved = await this.resolveUnsupportedExternal(file, type, {
          reason: 'missing-node-loader',
          error: missingNodeLoaderError,
          importOptions: options
        });
        if (resolved.resolved) {
          return resolved.module;
        }

        throw missingNodeLoaderError;
      }

      const normalized = cleanUrl(file);
      if (normalized.startsWith('file://')) {
        return this.nodeCommonJsLoader(fileUrlToPathWithFallback(normalized));
      }

      return this.nodeCommonJsLoader(normalized);
    }

    try {
      return await importModuleWithOptions(file, options);
    } catch (error) {
      if (!shouldTryImportFallback(error)) {
        throw error;
      }

      const resolved = await this.resolveUnsupportedExternal(file, type, {
        reason: 'import-failed',
        error,
        importOptions: options
      });
      if (resolved.resolved) {
        return resolved.module;
      }

      throw error;
    }
  }

  private async resolveUnsupportedExternal(
    file: string,
    type: ExternalFetchResult['type'],
    options?: {
      reason?: 'import-failed' | 'unsupported-external' | 'missing-node-loader';
      error?: unknown;
      importOptions?: unknown;
    }
  ) {
    if (!this.resolveExternalModule) {
      return { resolved: false };
    }

    const context = {
      id: file,
      type,
      evaluator: this.runtimeName,
      ...(options?.reason != null ? { reason: options.reason } : {}),
      ...(options?.error !== undefined ? { error: options.error } : {}),
      ...(options?.importOptions !== undefined
        ? { importOptions: options.importOptions }
        : {})
    };

    try {
      const resolved = await this.resolveExternalModule(context);
      return normalizeResolveResult(resolved);
    } catch (error) {
      const wrapped = new Error(
        `[farm module runner] resolveExternalModule failed for "${file}" (type=${type}, evaluator=${this.runtimeName}, reason=${options?.reason ?? 'unspecified'}).`
      );
      (wrapped as Error & { cause?: unknown }).cause = error;
      throw wrapped;
    }
  }

  private createNodeCommonJsLoader(): NodeRequireFunction | undefined {
    const createRequire = resolveNodeCreateRequire();

    if (!createRequire) {
      return undefined;
    }

    try {
      return createRequire(import.meta.url);
    } catch {
      return undefined;
    }
  }
}

export class ESModulesEvaluator extends BaseAsyncFunctionEvaluator {
  constructor(resolveExternalModule?: ExternalModuleResolver) {
    super('node', true, resolveExternalModule);
  }
}

export class WorkerModulesEvaluator extends BaseAsyncFunctionEvaluator {
  constructor(resolveExternalModule?: ExternalModuleResolver) {
    super('worker', false, resolveExternalModule);
  }
}

export class BunModulesEvaluator extends BaseAsyncFunctionEvaluator {
  constructor(resolveExternalModule?: ExternalModuleResolver) {
    super('bun', false, resolveExternalModule);
  }
}

export class DenoModulesEvaluator extends BaseAsyncFunctionEvaluator {
  constructor(resolveExternalModule?: ExternalModuleResolver) {
    super('deno', false, resolveExternalModule);
  }
}

export function detectHostEvaluatorType(
  globalLike: Record<string, unknown> = globalThis as unknown as Record<
    string,
    unknown
  >
): ModuleRunnerEvaluatorType {
  if (globalLike.Bun) {
    return 'bun';
  }

  if (globalLike.Deno) {
    return 'deno';
  }

  const processLike = Reflect.get(globalLike, 'process');
  const getBuiltinModule =
    processLike && typeof processLike === 'object'
      ? Reflect.get(processLike as object, 'getBuiltinModule')
      : undefined;

  if (typeof getBuiltinModule === 'function') {
    for (const moduleName of [
      'node:worker_threads',
      'worker_threads'
    ] as const) {
      try {
        const workerThreads = Reflect.apply(
          getBuiltinModule as (...args: unknown[]) => unknown,
          processLike,
          [moduleName]
        );
        const isMainThread =
          workerThreads && typeof workerThreads === 'object'
            ? Reflect.get(workerThreads as object, 'isMainThread')
            : undefined;

        if (isMainThread === false) {
          return 'worker';
        }
      } catch {
        // ignore and continue fallback detection
      }
    }
  }

  const workerGlobalScope = globalLike.WorkerGlobalScope;
  const selfRef = globalLike.self;

  if (
    typeof workerGlobalScope === 'function' &&
    typeof selfRef === 'object' &&
    selfRef != null &&
    selfRef instanceof workerGlobalScope
  ) {
    return 'worker';
  }

  return 'node';
}

export function createModuleEvaluator(
  evaluator?: ModuleRunnerEvaluatorType | ModuleEvaluator,
  resolveExternalModule?: ExternalModuleResolver
): ModuleEvaluator {
  if (evaluator && typeof evaluator === 'object') {
    return evaluator;
  }

  const evaluatorType = evaluator ?? detectHostEvaluatorType();

  switch (evaluatorType) {
    case 'worker':
      return new WorkerModulesEvaluator(resolveExternalModule);
    case 'bun':
      return new BunModulesEvaluator(resolveExternalModule);
    case 'deno':
      return new DenoModulesEvaluator(resolveExternalModule);
    case 'node':
    default:
      return new ESModulesEvaluator(resolveExternalModule);
  }
}
