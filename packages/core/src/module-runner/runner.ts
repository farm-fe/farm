import { fileURLToPath } from 'node:url';
import { cleanUrl } from '../utils/url.js';
import {
  farmSsrDynamicImportKey,
  farmSsrExportAllKey,
  farmSsrExportNameKey,
  farmSsrImportKey,
  farmSsrImportMetaKey,
  farmSsrModuleExportsKey
} from './constants.js';
import { createDefaultImportMeta } from './createImportMeta.js';
import { ModuleRunnerDiagnosticsBus } from './diagnostics.js';
import { EvaluatedModuleNode, EvaluatedModules } from './evaluatedModules.js';
import { createModuleEvaluator } from './evaluator.js';
import { createRunnerSourceMapInterceptor } from './sourceMapInterceptor.js';
import type {
  ExternalFetchResult,
  FarmModuleRunnerOptions,
  FetchResult,
  ModuleEvaluator,
  ModuleRunnerDiagnosticsEmitter,
  NonJsPolicyMode,
  NonJsPolicyOptions,
  ResolvedFetchResult,
  RunnerExternalPolicyContext,
  RunnerExternalPolicyMode,
  RunnerExternalPolicyTargetOptions,
  RunnerHotPayload,
  RunnerResolveContext,
  RunnerResolveResult,
  RunnerResolver,
  RunnerSourceMapHooks
} from './types.js';

type RuntimeImportKind = 'import' | 'dynamic-import';
type NonJsModuleKind = 'style' | 'asset' | 'unknown';

const STYLE_EXTENSIONS = new Set([
  '.css',
  '.less',
  '.sass',
  '.scss',
  '.styl',
  '.stylus',
  '.pcss',
  '.postcss'
]);

const ASSET_EXTENSIONS = new Set([
  '.png',
  '.jpg',
  '.jpeg',
  '.gif',
  '.svg',
  '.webp',
  '.avif',
  '.ico',
  '.bmp',
  '.tif',
  '.tiff',
  '.woff',
  '.woff2',
  '.ttf',
  '.otf',
  '.eot',
  '.mp4',
  '.webm',
  '.mp3',
  '.wav'
]);

const CJS_INTEROP_NON_THROW_MISSING_PROPS = new Set(['then']);

function createTraceId(prefix: string): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random()
    .toString(36)
    .slice(2, 8)}`;
}

function isObjectLike(
  value: unknown
): value is Record<string | symbol, unknown> {
  return (
    (typeof value === 'object' && value !== null) || typeof value === 'function'
  );
}

function isModuleNamespaceLike(
  value: unknown
): value is Record<string | symbol, unknown> {
  return (
    isObjectLike(value) && Reflect.get(value, Symbol.toStringTag) === 'Module'
  );
}

function createMissingDefaultExportError(params: {
  specifier: string;
  importer: string;
  kind: RuntimeImportKind;
}): Error {
  const error = new Error(
    `[farm module runner] Missing default export from "${params.specifier}" imported by "${params.importer}" via ${params.kind}.`
  ) as Error & {
    code?: string;
    specifier?: string;
    importer?: string;
  };
  error.code = 'ERR_MISSING_DEFAULT_EXPORT';
  error.specifier = params.specifier;
  error.importer = params.importer;
  return error;
}

function createMissingNamedExportError(params: {
  exportName: string;
  specifier: string;
  importer: string;
  kind: RuntimeImportKind;
}): Error {
  const error = new Error(
    `[farm module runner] Missing named export "${params.exportName}" from "${params.specifier}" imported by "${params.importer}" via ${params.kind}.`
  ) as Error & {
    code?: string;
    specifier?: string;
    importer?: string;
    exportName?: string;
  };
  error.code = 'ERR_MISSING_NAMED_EXPORT';
  error.specifier = params.specifier;
  error.importer = params.importer;
  error.exportName = params.exportName;
  return error;
}

function isVueStyleQuery(id: string): boolean {
  return /(?:\?|&)vue(?:&|$)/.test(id) && /(?:\?|&)type=style(?:&|$)/.test(id);
}

function getExtensionFromId(id: string): string {
  const normalized = cleanUrl(id);
  const lastSlash = normalized.lastIndexOf('/');
  const fileName =
    lastSlash >= 0 ? normalized.slice(lastSlash + 1) : normalized;
  const dot = fileName.lastIndexOf('.');
  return dot >= 0 ? fileName.slice(dot).toLowerCase() : '';
}

function toFilePathFromExternalize(externalize: string): string | null {
  const cleaned = cleanUrl(externalize);

  if (!cleaned.startsWith('file://')) {
    return null;
  }

  try {
    return fileURLToPath(cleaned);
  } catch {
    return null;
  }
}

function classifyNonJsKind(params: {
  id: string;
  externalize: string;
  bailoutReason?: string;
}): NonJsModuleKind | null {
  if (isVueStyleQuery(params.id)) {
    return 'style';
  }

  const idExt = getExtensionFromId(params.id);
  if (idExt && STYLE_EXTENSIONS.has(idExt)) {
    return 'style';
  }

  if (idExt && ASSET_EXTENSIONS.has(idExt)) {
    return 'asset';
  }

  const externalExt = getExtensionFromId(params.externalize);
  if (externalExt && STYLE_EXTENSIONS.has(externalExt)) {
    return 'style';
  }

  if (externalExt && ASSET_EXTENSIONS.has(externalExt)) {
    return 'asset';
  }

  if (params.bailoutReason === 'not-script') {
    return 'unknown';
  }

  return null;
}

function resolveNonJsPolicyMode(
  kind: NonJsModuleKind,
  policy?: NonJsPolicyOptions
): NonJsPolicyMode {
  if (kind === 'style') {
    return policy?.style ?? 'stub';
  }

  if (kind === 'asset') {
    return policy?.asset ?? 'stub';
  }

  return policy?.unknown ?? 'externalize';
}

export class FarmModuleRunner {
  readonly evaluatedModules: EvaluatedModules;
  private readonly sourceMapInterceptor: ReturnType<
    typeof createRunnerSourceMapInterceptor
  >;
  private readonly baseEvaluator: ModuleEvaluator;
  private readonly evaluator: ModuleEvaluator;
  private readonly resolver: RunnerResolver | undefined;
  private readonly diagnostics: ModuleRunnerDiagnosticsEmitter;
  private readonly namespaceInteropCache = new WeakMap<
    Record<string | symbol, unknown>,
    Record<string | symbol, unknown>
  >();
  private readonly cjsInteropCache = new WeakMap<
    Record<string | symbol, unknown>,
    Record<string | symbol, unknown>
  >();
  private readonly fetchInflight = new Map<string, Promise<FetchResult>>();
  private readonly moduleLastAccess = new Map<string, number>();
  private readonly gcCandidateQueue: string[] = [];
  private readonly gcCandidateSet = new Set<string>();
  private readonly policyEvictedModules = new Set<string>();
  private readonly reportedBailouts = new Set<string>();
  private readonly strictTransform: boolean;

  private closed = false;

  constructor(private readonly options: FarmModuleRunnerOptions) {
    this.strictTransform = options.strictTransform === true;

    this.baseEvaluator = createModuleEvaluator(
      options.evaluator,
      options.resolveExternalModule
    );
    this.evaluator = options.disableInterop
      ? this.baseEvaluator
      : this.createInteropEvaluator();
    this.resolver = options.resolver;
    this.diagnostics =
      options.diagnostics === false
        ? {
            emit() {},
            subscribe() {
              return () => undefined;
            },
            clear() {}
          }
        : (options.diagnostics ?? new ModuleRunnerDiagnosticsBus());

    this.sourceMapInterceptor = createRunnerSourceMapInterceptor(
      this.resolveSourceMapInterceptorEnabled(options.sourceMapInterceptor),
      this.resolveSourceMapInterceptorNative(options.sourceMapInterceptor),
      this.resolveSourceMapInterceptorHooks(options.sourceMapInterceptor)
    );

    this.evaluatedModules = options.evaluatedModules ?? new EvaluatedModules();

    if (options.hmr !== false) {
      if (!this.options.transport.connect) {
        throw new Error(
          '[farm module runner] HMR is enabled but transport.connect is not available.'
        );
      }

      void this.options.transport.connect({
        onMessage: (payload) => this.handleHotPayload(payload),
        onDisconnection: () => {
          // No-op for now. Keep room for future reconnect strategy.
        }
      });
    }
  }

  async import<T = unknown>(url: string): Promise<T> {
    const traceId = createTraceId('import');
    const resolved = await this.resolveRequest(url);
    const start = Date.now();
    this.diagnostics.emit({
      type: 'import:start',
      traceId,
      request: url,
      resolvedId: resolved.id,
      timestamp: start
    });

    try {
      const mod = await this.cachedModule(resolved.id);
      const result = (await this.cachedRequest(resolved.id, mod)) as T;
      this.diagnostics.emit({
        type: 'import:end',
        traceId,
        request: url,
        moduleId: resolved.id,
        durationMs: Date.now() - start,
        timestamp: Date.now()
      });
      return result;
    } catch (error) {
      this.diagnostics.emit({
        type: 'import:error',
        traceId,
        request: url,
        resolvedId: resolved.id,
        error,
        timestamp: Date.now()
      });
      throw error;
    }
  }

  clearCache(): void {
    this.sourceMapInterceptor.clear();
    this.evaluatedModules.clear();
    this.fetchInflight.clear();
    this.moduleLastAccess.clear();
    this.gcCandidateQueue.length = 0;
    this.gcCandidateSet.clear();
    this.policyEvictedModules.clear();
    this.reportedBailouts.clear();
    this.diagnostics.clear();
  }

  async close(): Promise<void> {
    if (this.closed) {
      return;
    }

    this.closed = true;
    this.clearCache();
    this.sourceMapInterceptor.close();
    await this.options.transport.disconnect?.();
  }

  isClosed(): boolean {
    return this.closed;
  }

  private async cachedModule(url: string, importer?: string) {
    const cached = this.evaluatedModules.getModuleByUrl(url);
    const expired = Boolean(cached && this.isModuleExpired(cached));
    if (cached && expired) {
      this.invalidateModuleForGc(cached);
    }

    const hasUsableMeta = Boolean(
      cached && !expired && !this.policyEvictedModules.has(cached.id)
    );
    const fetchTraceId = createTraceId('fetch');
    const fetchStart = Date.now();
    this.diagnostics.emit({
      type: 'fetch:start',
      traceId: fetchTraceId,
      request: url,
      ...(importer ? { importer } : {}),
      cached: hasUsableMeta,
      timestamp: fetchStart
    });
    const inflightKey = `${importer ?? '<entry>'}::${url}`;
    const existing = this.fetchInflight.get(inflightKey);
    const fetchPromise =
      existing ??
      (this.options.transport.invoke('fetchModule', [
        url,
        importer,
        {
          cached: hasUsableMeta,
          startOffset: this.evaluator.startOffset
        }
      ]) as Promise<FetchResult>);

    if (!existing) {
      this.fetchInflight.set(inflightKey, fetchPromise);
    }

    let fetchedRaw: FetchResult;
    try {
      fetchedRaw = await fetchPromise;
    } catch (error) {
      this.diagnostics.emit({
        type: 'fetch:error',
        traceId: fetchTraceId,
        request: url,
        ...(importer ? { importer } : {}),
        cached: hasUsableMeta,
        error,
        timestamp: Date.now()
      });
      throw error;
    } finally {
      if (!existing) {
        this.fetchInflight.delete(inflightKey);
      }
    }
    const fetched = this.applyNonJsPolicy(fetchedRaw, url);
    this.diagnostics.emit({
      type: 'fetch:end',
      traceId: fetchTraceId,
      request: url,
      ...(importer ? { importer } : {}),
      fromCache: 'cache' in fetched,
      resultKind:
        'cache' in fetched
          ? 'cache'
          : 'externalize' in fetched
            ? 'external'
            : 'inlined',
      durationMs: Date.now() - fetchStart,
      timestamp: Date.now()
    });

    if ('cache' in fetched) {
      if (!cached?.meta) {
        throw new Error(
          `[farm module runner] Module "${url}" returned cache=true before first load.`
        );
      }

      this.runIncrementalGc();
      return cached;
    }

    const moduleId =
      'externalize' in fetched ? fetched.externalize : fetched.id;
    const moduleUrl = 'url' in fetched ? fetched.url : url;
    const mod = this.evaluatedModules.ensureModule(moduleId, moduleUrl);

    if ('invalidate' in fetched && fetched.invalidate) {
      this.invalidateModuleForGc(mod);
    }

    const normalized: ResolvedFetchResult = {
      ...fetched,
      id: moduleId,
      url: moduleUrl
    };

    mod.meta = normalized;
    this.policyEvictedModules.delete(mod.id);
    this.markModuleAccess(mod.id);
    this.enforceCachePolicy(mod.id);
    this.runIncrementalGc();

    if ('externalize' in normalized && normalized.bailoutReason) {
      if (this.strictTransform) {
        throw new Error(
          `[farm module runner] Module "${normalized.id}" fallback externalized with bailoutReason="${normalized.bailoutReason}" (strictTransform=true).`
        );
      }

      const key = `${normalized.id}::${normalized.bailoutReason}`;
      if (!this.reportedBailouts.has(key)) {
        this.reportedBailouts.add(key);
        console.warn(
          `[farm module runner] Module "${normalized.id}" fallback externalized with bailoutReason="${normalized.bailoutReason}".`
        );
      }
    }

    if ('code' in normalized && normalized.map) {
      const sourceId = normalized.file || normalized.id || normalized.url;
      this.sourceMapInterceptor.register(sourceId, normalized.map);
    }

    return mod;
  }

  private async cachedRequest(
    url: string,
    mod: ReturnType<EvaluatedModules['ensureModule']>,
    callstack: string[] = [],
    importOptions?: unknown
  ): Promise<unknown> {
    const meta = mod.meta;

    if (!meta) {
      throw new Error(
        `[farm module runner] Missing module metadata for "${url}".`
      );
    }

    const moduleId = meta.id;
    this.markModuleAccess(moduleId);

    if (mod.evaluated && mod.promise) {
      return mod.promise;
    }

    if (callstack.includes(moduleId) && mod.exports) {
      return mod.exports;
    }

    if (mod.promise) {
      return mod.promise;
    }

    const promise = this.directRequest(url, mod, callstack, importOptions);
    mod.promise = promise;
    mod.evaluated = false;

    try {
      return await promise;
    } finally {
      mod.evaluated = true;
    }
  }

  private async directRequest(
    url: string,
    mod: ReturnType<EvaluatedModules['ensureModule']>,
    callstack: string[],
    importOptions?: unknown
  ): Promise<unknown> {
    const meta = mod.meta;

    if (!meta) {
      throw new Error(
        `[farm module runner] Missing module metadata for "${url}".`
      );
    }

    const request = async (
      dep: string,
      depImportOptions?: unknown
    ): Promise<unknown> => {
      dep = String(dep);
      const requestBase = meta.url || url;
      const resolvedDep = await this.resolveRequest(dep, {
        importer: requestBase
      });

      const importer = meta.id;
      const depMod = await this.cachedModule(resolvedDep.id, importer);
      depMod.importers.add(importer);
      mod.imports.add(depMod.id);

      return this.cachedRequest(
        resolvedDep.id,
        depMod,
        [...callstack, importer],
        depImportOptions
      );
    };

    const dynamicRequest = async (
      dep: string,
      options?: unknown
    ): Promise<unknown> => {
      return request(dep, options);
    };

    if ('externalize' in meta) {
      const policyResolved = await this.resolveExternalByPolicy(
        url,
        meta.externalize,
        meta.type
      );
      if (policyResolved.resolved) {
        mod.exports = policyResolved.module;
        return policyResolved.module;
      }

      const evalTraceId = createTraceId('eval');
      const evalStart = Date.now();
      this.diagnostics.emit({
        type: 'eval:start',
        traceId: evalTraceId,
        moduleId: meta.id,
        mode: 'external',
        timestamp: evalStart
      });
      try {
        const exports = await this.evaluator.runExternalModule(
          meta.externalize,
          meta.type,
          importOptions
        );
        mod.exports = exports;
        this.diagnostics.emit({
          type: 'eval:end',
          traceId: evalTraceId,
          moduleId: meta.id,
          mode: 'external',
          durationMs: Date.now() - evalStart,
          timestamp: Date.now()
        });
        return exports;
      } catch (error) {
        this.diagnostics.emit({
          type: 'eval:error',
          traceId: evalTraceId,
          moduleId: meta.id,
          mode: 'external',
          error,
          timestamp: Date.now()
        });
        throw error;
      }
    }

    if (!meta.code) {
      throw new Error(`[farm module runner] Missing code for "${url}".`);
    }

    const createImportMeta =
      this.options.createImportMeta ?? createDefaultImportMeta;
    const modulePath = meta.file || meta.id;
    const baseImportMeta = await createImportMeta(modulePath);
    const importMeta = {
      ...baseImportMeta,
      resolve: (request: string) =>
        this.resolveImportMetaRequestSync(request, modulePath)
    };
    const exports: Record<string, unknown> = Object.create(null);

    Object.defineProperty(exports, Symbol.toStringTag, {
      value: 'Module',
      enumerable: false,
      configurable: false
    });

    mod.exports = exports;

    const evalTraceId = createTraceId('eval');
    const evalStart = Date.now();
    this.diagnostics.emit({
      type: 'eval:start',
      traceId: evalTraceId,
      moduleId: meta.id,
      mode: 'inlined',
      timestamp: evalStart
    });
    try {
      await this.evaluator.runInlinedModule(
        {
          [farmSsrModuleExportsKey]: exports,
          [farmSsrImportKey]: request,
          [farmSsrDynamicImportKey]: dynamicRequest,
          [farmSsrExportAllKey]: (source) => this.exportAll(exports, source),
          [farmSsrExportNameKey]: (name, getter) => {
            Object.defineProperty(exports, name, {
              enumerable: true,
              configurable: true,
              get: getter
            });
          },
          [farmSsrImportMetaKey]: importMeta
        },
        meta.code,
        mod
      );
      this.diagnostics.emit({
        type: 'eval:end',
        traceId: evalTraceId,
        moduleId: meta.id,
        mode: 'inlined',
        durationMs: Date.now() - evalStart,
        timestamp: Date.now()
      });
    } catch (error) {
      this.diagnostics.emit({
        type: 'eval:error',
        traceId: evalTraceId,
        moduleId: meta.id,
        mode: 'inlined',
        error,
        timestamp: Date.now()
      });
      throw error;
    }

    return exports;
  }

  private async resolveExternalByPolicy(
    requestId: string,
    externalize: string,
    type: ExternalFetchResult['type']
  ): Promise<{ resolved: boolean; module?: unknown }> {
    const policy = this.options.externalPolicy;
    const policyTraceId = createTraceId('policy');
    const globals = policy?.globals;
    const candidates = [
      requestId,
      cleanUrl(requestId),
      externalize,
      cleanUrl(externalize)
    ];

    if (globals) {
      if (typeof globals === 'function') {
        for (const id of candidates) {
          const resolved = await globals(id);
          if (resolved !== undefined) {
            this.diagnostics.emit({
              type: 'external:policy',
              traceId: policyTraceId,
              requestId,
              externalize,
              externalType: type,
              policy: 'globals',
              action: 'resolved',
              timestamp: Date.now()
            });
            return { resolved: true, module: resolved };
          }
        }
      } else {
        for (const id of candidates) {
          if (Object.hasOwn(globals, id)) {
            this.diagnostics.emit({
              type: 'external:policy',
              traceId: policyTraceId,
              requestId,
              externalize,
              externalType: type,
              policy: 'globals',
              action: 'resolved',
              timestamp: Date.now()
            });
            return {
              resolved: true,
              module: globals[id]
            };
          }
        }
      }
    }

    const context: RunnerExternalPolicyContext = {
      requestId,
      externalize,
      type
    };

    if (policy?.custom) {
      const customResolved = await policy.custom(context);
      if (customResolved !== undefined) {
        this.diagnostics.emit({
          type: 'external:policy',
          traceId: policyTraceId,
          requestId,
          externalize,
          externalType: type,
          policy: 'custom',
          action: 'resolved',
          timestamp: Date.now()
        });
        return { resolved: true, module: customResolved };
      }
    }

    const targetPolicy = this.resolveExternalTargetPolicy(type, policy);
    const targetPolicyName = type === 'builtin' ? 'builtin' : 'network';
    if (!targetPolicy || targetPolicy.mode === 'externalize') {
      if (targetPolicy) {
        this.diagnostics.emit({
          type: 'external:policy',
          traceId: policyTraceId,
          requestId,
          externalize,
          externalType: type,
          policy: targetPolicyName,
          action: 'externalize',
          timestamp: Date.now()
        });
      }
      return { resolved: false };
    }

    if (targetPolicy.mode === 'error') {
      const messageValue = targetPolicy.message;
      const message =
        typeof messageValue === 'function'
          ? await messageValue(context)
          : messageValue;
      this.diagnostics.emit({
        type: 'external:policy',
        traceId: policyTraceId,
        requestId,
        externalize,
        externalType: type,
        policy: targetPolicyName,
        action: 'error',
        timestamp: Date.now()
      });

      throw new Error(
        message ??
          `[farm module runner] External module "${requestId}" (${type}) is blocked by externalPolicy.${type}=error.`
      );
    }

    const stubValue = targetPolicy.stub;
    if (typeof stubValue === 'function') {
      this.diagnostics.emit({
        type: 'external:policy',
        traceId: policyTraceId,
        requestId,
        externalize,
        externalType: type,
        policy: targetPolicyName,
        action: 'stub',
        timestamp: Date.now()
      });
      return {
        resolved: true,
        module: await stubValue(context)
      };
    }

    if (stubValue !== undefined) {
      this.diagnostics.emit({
        type: 'external:policy',
        traceId: policyTraceId,
        requestId,
        externalize,
        externalType: type,
        policy: targetPolicyName,
        action: 'stub',
        timestamp: Date.now()
      });
      return { resolved: true, module: stubValue };
    }

    this.diagnostics.emit({
      type: 'external:policy',
      traceId: policyTraceId,
      requestId,
      externalize,
      externalType: type,
      policy: targetPolicyName,
      action: 'stub',
      timestamp: Date.now()
    });
    return {
      resolved: true,
      module: this.createDefaultExternalPolicyStub(context)
    };
  }

  private resolveExternalTargetPolicy(
    type: ExternalFetchResult['type'],
    policy: FarmModuleRunnerOptions['externalPolicy']
  ): RunnerExternalPolicyTargetOptions | undefined {
    const raw =
      type === 'builtin'
        ? policy?.builtin
        : type === 'network'
          ? policy?.network
          : undefined;

    if (!raw) {
      return undefined;
    }

    if (typeof raw === 'string') {
      return { mode: raw as RunnerExternalPolicyMode };
    }

    return {
      mode: raw.mode ?? 'externalize',
      ...(raw.stub !== undefined ? { stub: raw.stub } : {}),
      ...(raw.message !== undefined ? { message: raw.message } : {})
    };
  }

  private createDefaultExternalPolicyStub(
    context: RunnerExternalPolicyContext
  ): unknown {
    if (context.type === 'network') {
      return cleanUrl(context.externalize);
    }

    return Object.create(null);
  }

  private isModuleExpired(
    mod: ReturnType<EvaluatedModules['ensureModule']>
  ): boolean {
    const ttlMs = this.options.cachePolicy?.ttlMs;
    if (!ttlMs || ttlMs <= 0) {
      return false;
    }

    if (!mod.meta) {
      return false;
    }

    const last = this.moduleLastAccess.get(mod.id);
    if (last == null) {
      return false;
    }

    return Date.now() - last > ttlMs;
  }

  private markModuleAccess(moduleId: string): void {
    this.moduleLastAccess.set(moduleId, Date.now());
  }

  private enforceCachePolicy(currentModuleId: string): void {
    const maxEntries = this.options.cachePolicy?.maxEntries;
    if (!maxEntries || maxEntries <= 0) {
      return;
    }

    const active = [...this.evaluatedModules.idToModuleMap.values()].filter(
      (mod) => Boolean(mod.meta)
    );

    if (active.length <= maxEntries) {
      return;
    }

    const candidates = active
      .filter(
        (mod) => mod.id !== currentModuleId && !(mod.promise && !mod.evaluated)
      )
      .sort((a, b) => {
        const aTs = this.moduleLastAccess.get(a.id) ?? 0;
        const bTs = this.moduleLastAccess.get(b.id) ?? 0;
        return aTs - bTs;
      });

    let overflow = active.length - maxEntries;
    for (const mod of candidates) {
      if (overflow <= 0) {
        break;
      }

      this.invalidateModuleForGc(mod);
      this.moduleLastAccess.delete(mod.id);
      this.policyEvictedModules.add(mod.id);
      overflow--;
    }
  }

  private invalidateModuleForGc(mod: EvaluatedModuleNode): void {
    this.evaluatedModules.invalidateModule(mod);
    this.scheduleGcCandidate(mod.id);
  }

  private scheduleGcCandidate(moduleId: string): void {
    if (this.gcCandidateSet.has(moduleId)) {
      return;
    }

    this.gcCandidateSet.add(moduleId);
    this.gcCandidateQueue.push(moduleId);
  }

  private runIncrementalGc(): void {
    const budget = this.options.cachePolicy?.gcSweepPerCycle ?? 16;
    if (budget <= 0) {
      return;
    }

    let remaining = budget;
    while (remaining > 0 && this.gcCandidateQueue.length) {
      const moduleId = this.gcCandidateQueue.shift();
      if (!moduleId) {
        break;
      }
      this.gcCandidateSet.delete(moduleId);

      const mod = this.evaluatedModules.getModuleById(moduleId);
      if (!mod) {
        remaining--;
        continue;
      }

      if (
        !mod.meta &&
        !mod.promise &&
        mod.imports.size === 0 &&
        mod.importers.size === 0
      ) {
        this.evaluatedModules.removeModule(mod);
        this.moduleLastAccess.delete(moduleId);
        this.policyEvictedModules.delete(moduleId);
      } else if (!mod.meta) {
        this.scheduleGcCandidate(moduleId);
      }

      remaining--;
    }
  }

  private applyNonJsPolicy(result: FetchResult, moduleId: string): FetchResult {
    if (!('externalize' in result)) {
      return result;
    }

    const kind = classifyNonJsKind({
      id: moduleId,
      externalize: result.externalize,
      bailoutReason: result.bailoutReason
    });

    if (!kind) {
      return result;
    }

    const mode = resolveNonJsPolicyMode(kind, this.options.nonJsPolicy);

    if (mode === 'externalize') {
      return result;
    }

    if (mode === 'error') {
      throw new Error(
        `[farm module runner] Non-js module "${moduleId}" (${kind}) is blocked by nonJsPolicy=${mode}.`
      );
    }

    const file = toFilePathFromExternalize(result.externalize);

    if (kind === 'asset') {
      const value = JSON.stringify(cleanUrl(result.externalize));
      return {
        code: [
          `const __farm_ssr_asset_url__ = ${value};`,
          '__farm_ssr_export_name__("default", () => __farm_ssr_asset_url__);'
        ].join('\n'),
        file,
        id: moduleId,
        url: moduleId,
        invalidate: false,
        map: null
      };
    }

    return {
      code: [
        'const __farm_ssr_style_noop__ = {};',
        '__farm_ssr_export_name__("default", () => __farm_ssr_style_noop__);'
      ].join('\n'),
      file,
      id: moduleId,
      url: moduleId,
      invalidate: false,
      map: null
    };
  }

  private createInteropEvaluator(): ModuleEvaluator {
    return {
      startOffset: this.baseEvaluator.startOffset,
      runExternalModule: (file, type, options) =>
        this.baseEvaluator.runExternalModule(file, type, options),
      runInlinedModule: async (context, code, module) => {
        const wrapImport =
          (
            loader: (id: string, options?: unknown) => Promise<unknown>,
            kind: RuntimeImportKind
          ) =>
          async (id: string, options?: unknown) => {
            const loaded = await loader(id, options);
            return this.applyInteropForImportedModule(loaded, {
              specifier: String(id),
              importer: module.id,
              kind
            });
          };

        const wrappedContext = {
          ...context,
          __farm_ssr_import__: wrapImport(
            context.__farm_ssr_import__,
            'import'
          ),
          __farm_ssr_dynamic_import__: wrapImport(
            context.__farm_ssr_dynamic_import__,
            'dynamic-import'
          )
        };

        await this.baseEvaluator.runInlinedModule(wrappedContext, code, module);
      }
    };
  }

  private applyInteropForImportedModule(
    loaded: unknown,
    info: {
      specifier: string;
      importer: string;
      kind: RuntimeImportKind;
    }
  ): unknown {
    const traceId = createTraceId('interop');

    if (!isObjectLike(loaded)) {
      return loaded;
    }

    if (isModuleNamespaceLike(loaded)) {
      const cached = this.namespaceInteropCache.get(loaded);
      if (cached) {
        return cached;
      }

      const wrapped = new Proxy(loaded, {
        get: (target, prop, receiver) => {
          if (prop === 'default' && !Reflect.has(target, 'default')) {
            const error = createMissingDefaultExportError(info);
            this.diagnostics.emit({
              type: 'interop:error',
              traceId,
              specifier: info.specifier,
              importer: info.importer,
              code: 'ERR_MISSING_DEFAULT_EXPORT',
              message: error.message,
              timestamp: Date.now()
            });
            throw error;
          }

          return Reflect.get(target, prop, receiver);
        }
      });

      this.diagnostics.emit({
        type: 'interop:wrap',
        traceId,
        specifier: info.specifier,
        importer: info.importer,
        kind: 'esm-namespace',
        timestamp: Date.now()
      });
      this.namespaceInteropCache.set(loaded, wrapped);
      return wrapped;
    }

    const cached = this.cjsInteropCache.get(loaded);
    if (cached) {
      return cached;
    }

    const cjsSource = loaded;
    const wrapped = new Proxy(Object.create(null) as Record<string, unknown>, {
      get: (_target, prop) => {
        if (prop === Symbol.toStringTag) {
          return 'Module';
        }

        if (prop === '__esModule') {
          return true;
        }

        if (prop === 'default') {
          return cjsSource;
        }

        if (
          typeof prop === 'string' &&
          !CJS_INTEROP_NON_THROW_MISSING_PROPS.has(prop) &&
          !Reflect.has(cjsSource, prop)
        ) {
          const error = createMissingNamedExportError({
            exportName: prop,
            ...info
          });
          this.diagnostics.emit({
            type: 'interop:error',
            traceId,
            specifier: info.specifier,
            importer: info.importer,
            code: 'ERR_MISSING_NAMED_EXPORT',
            message: error.message,
            timestamp: Date.now()
          });
          throw error;
        }

        return Reflect.get(cjsSource, prop, cjsSource);
      },
      has: (_target, prop) => {
        if (prop === '__esModule' || prop === 'default') {
          return true;
        }
        return Reflect.has(cjsSource, prop);
      },
      ownKeys: () => {
        const keys = new Set<string | symbol>([
          ...Reflect.ownKeys(cjsSource),
          'default',
          '__esModule'
        ]);

        return Array.from(keys);
      },
      getOwnPropertyDescriptor: (_target, prop) => {
        if (prop === 'default') {
          return {
            configurable: true,
            enumerable: true,
            writable: false,
            value: cjsSource
          };
        }

        if (prop === '__esModule') {
          return {
            configurable: true,
            enumerable: false,
            writable: false,
            value: true
          };
        }

        return Reflect.getOwnPropertyDescriptor(cjsSource, prop);
      }
    });

    this.diagnostics.emit({
      type: 'interop:wrap',
      traceId,
      specifier: info.specifier,
      importer: info.importer,
      kind: 'cjs-value',
      timestamp: Date.now()
    });
    this.cjsInteropCache.set(loaded, wrapped);
    return wrapped;
  }

  private async resolveRequest(
    request: string,
    context?: RunnerResolveContext
  ): Promise<RunnerResolveResult> {
    const traceId = createTraceId('resolve');
    const start = Date.now();
    this.diagnostics.emit({
      type: 'resolve:start',
      traceId,
      request,
      ...(context?.importer ? { importer: context.importer } : {}),
      timestamp: start
    });

    let resolved: RunnerResolveResult;

    if (this.resolver) {
      resolved = await this.resolver(request, context);
    } else if (request[0] === '.' && context?.importer) {
      resolved = {
        id: resolveRelativeRequestUrl(context.importer, request)
      };
    } else {
      resolved = { id: request };
    }

    const end = Date.now();
    this.diagnostics.emit({
      type: 'resolve:end',
      traceId,
      request,
      ...(context?.importer ? { importer: context.importer } : {}),
      resolvedId: resolved.id,
      resolvedUrl: resolved.url ?? resolved.id,
      durationMs: end - start,
      timestamp: end
    });

    return resolved;
  }

  private resolveImportMetaRequestSync(
    request: string,
    importer: string
  ): string {
    if (this.resolver) {
      const resolved = this.resolver(request, {
        importer
      });

      if (
        resolved &&
        typeof resolved === 'object' &&
        typeof Reflect.get(resolved as object, 'then') === 'function'
      ) {
        throw new Error(
          '[farm module runner] import.meta.resolve requires a synchronous resolver result.'
        );
      }

      const normalized = resolved as RunnerResolveResult;
      return normalized.url ?? normalized.id;
    }

    if (request[0] === '.') {
      return resolveRelativeRequestUrl(importer, request);
    }

    return request;
  }

  private exportAll(
    exports: Record<string, unknown>,
    sourceModule: unknown
  ): void {
    if (
      !sourceModule ||
      typeof sourceModule !== 'object' ||
      Array.isArray(sourceModule) ||
      sourceModule instanceof Promise ||
      sourceModule === exports
    ) {
      return;
    }

    for (const key in sourceModule as Record<string, unknown>) {
      if (key === 'default' || key === '__esModule' || key in exports) {
        continue;
      }

      Object.defineProperty(exports, key, {
        enumerable: true,
        configurable: true,
        get: () => (sourceModule as Record<string, unknown>)[key]
      });
    }
  }

  private handleHotPayload(payload: RunnerHotPayload): void {
    if (this.closed) {
      return;
    }

    if (payload.type === 'update') {
      this.diagnostics.emit({
        type: 'hmr:update',
        updates: payload.updates.length,
        timestamp: Date.now()
      });
      for (const update of payload.updates) {
        this.invalidateByUrl(update.path);
        this.invalidateByUrl(update.acceptedPath);
      }
      return;
    }

    if (payload.type === 'full-reload') {
      this.diagnostics.emit({
        type: 'hmr:full-reload',
        ...(payload.path ? { path: payload.path } : {}),
        timestamp: Date.now()
      });
      this.clearCache();
      return;
    }

    if (payload.type === 'prune') {
      this.diagnostics.emit({
        type: 'hmr:prune',
        paths: payload.paths.length,
        timestamp: Date.now()
      });
      for (const path of payload.paths) {
        this.invalidateByUrl(path);
      }
    }
  }

  private invalidateByUrl(url: string): void {
    const normalized = cleanUrl(url);
    const roots = new Set<EvaluatedModuleNode>();

    const directByRaw = this.evaluatedModules.getModuleByUrl(url);
    if (directByRaw) {
      roots.add(directByRaw);
    }

    const directByNormalized = this.evaluatedModules.getModuleByUrl(normalized);
    if (directByNormalized) {
      roots.add(directByNormalized);
    }

    const fileMatched = this.evaluatedModules.getModulesByFile(normalized);
    if (fileMatched) {
      for (const mod of fileMatched) {
        roots.add(mod);
      }
    }

    if (!roots.size) {
      return;
    }

    const queue = [...roots];
    const visited = new Set<string>();

    while (queue.length) {
      const mod = queue.pop();

      if (!mod || visited.has(mod.id)) {
        continue;
      }

      visited.add(mod.id);
      const importers = [...mod.importers];
      const meta = mod.meta;

      if (meta && 'code' in meta && meta.map) {
        const sourceId = meta.file || meta.id || meta.url;
        this.sourceMapInterceptor.unregister(sourceId);
      }

      this.invalidateModuleForGc(mod);

      for (const importerId of importers) {
        const importer = this.evaluatedModules.getModuleById(importerId);

        if (importer && !visited.has(importer.id)) {
          queue.push(importer);
        }
      }
    }
  }

  private resolveSourceMapInterceptorEnabled(
    option: FarmModuleRunnerOptions['sourceMapInterceptor']
  ): boolean {
    if (option == null) {
      return true;
    }

    if (typeof option === 'boolean') {
      return option;
    }

    return option.enable !== false;
  }

  private resolveSourceMapInterceptorNative(
    option: FarmModuleRunnerOptions['sourceMapInterceptor']
  ): boolean {
    if (option == null || typeof option === 'boolean') {
      return true;
    }

    return option.native !== false;
  }

  private resolveSourceMapInterceptorHooks(
    option: FarmModuleRunnerOptions['sourceMapInterceptor']
  ): RunnerSourceMapHooks | undefined {
    if (option == null || typeof option === 'boolean') {
      return undefined;
    }

    return option.hooks;
  }
}

function resolveRelativeRequestUrl(
  importerUrl: string,
  request: string
): string {
  const base = cleanUrl(importerUrl);

  if (
    base.startsWith('file://') ||
    base.startsWith('http://') ||
    base.startsWith('https://')
  ) {
    try {
      return new URL(request, base).toString();
    } catch {
      return request;
    }
  }

  const normalizedBase = base.startsWith('/') ? base : `/${base}`;

  try {
    return new URL(request, `http://farm.invalid${normalizedBase}`).pathname;
  } catch {
    return request;
  }
}
