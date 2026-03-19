import type {
  EvaluatedModuleNode,
  EvaluatedModules
} from './evaluatedModules.js';

export interface FarmRunnerImportMeta extends ImportMeta {
  url: string;
  env: Record<string, unknown>;
  [key: string]: unknown;
}

export interface FarmRunnerContext {
  __farm_ssr_exports__: Record<string, unknown>;
  __farm_ssr_import__: (id: string, options?: unknown) => Promise<unknown>;
  __farm_ssr_dynamic_import__: (
    id: string,
    options?: unknown
  ) => Promise<unknown>;
  __farm_ssr_export_all__: (obj: unknown) => void;
  __farm_ssr_export_name__: (name: string, getter: () => unknown) => void;
  __farm_ssr_import_meta__: FarmRunnerImportMeta;
}

export interface ModuleEvaluator {
  startOffset?: number;
  runInlinedModule(
    context: FarmRunnerContext,
    code: string,
    module: Readonly<EvaluatedModuleNode>
  ): Promise<void>;
  runExternalModule(
    file: string,
    type?: ExternalFetchResult['type'],
    options?: unknown
  ): Promise<unknown>;
}

export type ModuleRunnerEvaluatorType = 'node' | 'worker' | 'bun' | 'deno';

export interface FetchFunctionOptions {
  cached?: boolean;
  startOffset?: number;
}

export interface RunnerResolveContext {
  importer?: string;
}

export interface RunnerResolveResult {
  id: string;
  url?: string;
}

export type RunnerResolver = (
  request: string,
  context?: RunnerResolveContext
) => RunnerResolveResult | Promise<RunnerResolveResult>;

export type RunnerExternalPolicyMode = 'externalize' | 'stub' | 'error';

export interface RunnerExternalPolicyContext {
  requestId: string;
  externalize: string;
  type: 'module' | 'commonjs' | 'builtin' | 'network';
}

export interface RunnerExternalPolicyTargetOptions {
  mode?: RunnerExternalPolicyMode;
  stub?:
    | unknown
    | ((context: RunnerExternalPolicyContext) => unknown | Promise<unknown>);
  message?:
    | string
    | ((context: RunnerExternalPolicyContext) => string | Promise<string>);
}

export interface RunnerExternalPolicyOptions {
  globals?:
    | Record<string, unknown>
    | ((id: string) => unknown | undefined | Promise<unknown | undefined>);
  builtin?: RunnerExternalPolicyMode | RunnerExternalPolicyTargetOptions;
  network?: RunnerExternalPolicyMode | RunnerExternalPolicyTargetOptions;
  custom?: (
    context: RunnerExternalPolicyContext
  ) => unknown | undefined | Promise<unknown | undefined>;
}

export interface RunnerCachePolicyOptions {
  /**
   * Max active module entries kept in runner cache.
   * Older entries are invalidated in LRU order when exceeded.
   */
  maxEntries?: number;
  /**
   * Time-to-live for cached module metadata.
   * Expired entries are invalidated before fetch.
   */
  ttlMs?: number;
  /**
   * Incremental GC sweep budget per cycle.
   * Collected only from previously invalidated candidates.
   * Default is 16.
   */
  gcSweepPerCycle?: number;
}

export type NonJsPolicyMode = 'stub' | 'externalize' | 'error';

export interface NonJsPolicyOptions {
  style?: NonJsPolicyMode;
  asset?: NonJsPolicyMode;
  unknown?: NonJsPolicyMode;
}

export interface CachedFetchResult {
  cache: true;
}

export interface ExternalFetchResult {
  bailoutReason?: RunnerTransformBailoutReason;
  externalize: string;
  type: 'module' | 'commonjs' | 'builtin' | 'network';
}

export type RunnerTransformBailoutReason =
  | 'unsupported-ts'
  | 'import-mutation'
  | 'unhandled-module-decl'
  | 'not-script'
  | 'empty-content';

export interface ExternalModuleResolveContext {
  id: string;
  type: ExternalFetchResult['type'];
  evaluator: ModuleRunnerEvaluatorType;
  reason?: 'import-failed' | 'unsupported-external' | 'missing-node-loader';
  error?: unknown;
  importOptions?: unknown;
}

export interface ExternalModuleResolveResult {
  resolved: boolean;
  module?: unknown;
}

export type ExternalModuleResolver = (
  context: ExternalModuleResolveContext
) => ExternalModuleResolveResult | Promise<ExternalModuleResolveResult>;

export interface InlinedFetchResult {
  code: string;
  file: string | null;
  id: string;
  url: string;
  invalidate: boolean;
  map?: string | null;
}

export type FetchResult =
  | CachedFetchResult
  | ExternalFetchResult
  | InlinedFetchResult;

export type InvokeMethods = {
  fetchModule: (
    id: string,
    importer?: string,
    options?: FetchFunctionOptions
  ) => Promise<FetchResult>;
  getBuiltins: () => Promise<string[]>;
};

export interface ModuleRunnerInvokeCompiler {
  compiling: boolean;
  waitForCompileFinish: () => Promise<void>;
  fetchModule: (
    id: string,
    importer?: string,
    options?: FetchFunctionOptions
  ) => unknown;
  resource: (name: string) => unknown;
  resources: () => Record<string, string | Buffer>;
}

export interface ModuleRunnerInvokeContext {
  root: string;
  publicPath: string;
  moduleRunnerStamp: number;
  compiler: ModuleRunnerInvokeCompiler | null | undefined;
}

export type RunnerInvokePayload = {
  id: 'send' | `send:${string}`;
  name: keyof InvokeMethods;
  data: unknown[];
};

export type RunnerInvokeResponsePayload = {
  id: 'response' | `response:${string}`;
  name: keyof InvokeMethods;
  data: { result: unknown } | { error: unknown };
};

export interface RunnerHotUpdate {
  type: 'js-update';
  path: string;
  acceptedPath: string;
  timestamp: number;
}

export type RunnerHotPayload =
  | { type: 'connected' }
  | { type: 'update'; updates: RunnerHotUpdate[] }
  | { type: 'full-reload'; path?: string }
  | { type: 'custom'; event: string; data?: unknown }
  | {
      type: 'error';
      err: {
        message: string;
        stack?: string;
      };
    }
  | { type: 'prune'; paths: string[] };

export interface ModuleRunnerTransport {
  connect?(handlers: {
    onMessage: (payload: RunnerHotPayload) => void;
    onDisconnection: () => void;
  }): Promise<void> | void;
  disconnect?(): Promise<void> | void;
  invoke<T extends keyof InvokeMethods>(
    name: T,
    data: Parameters<InvokeMethods[T]>
  ): Promise<Awaited<ReturnType<InvokeMethods[T]>>>;
}

export interface RunnerSourceMapInterceptorOptions {
  enable?: boolean;
  native?: boolean;
  hooks?: RunnerSourceMapHooks;
}

export interface RunnerSourceMapHooks {
  retrieveSourceMap?: (source: string) => string | null | undefined;
  retrieveFile?: (source: string) => string | null | undefined;
  formatStack?: (params: {
    error: Error;
    remappedStack: string;
    trace: NodeJS.CallSite[];
  }) => unknown;
}

export type ModuleRunnerDiagnosticsEvent =
  | {
      type: 'resolve:start';
      traceId: string;
      request: string;
      importer?: string;
      timestamp: number;
    }
  | {
      type: 'resolve:end';
      traceId: string;
      request: string;
      importer?: string;
      resolvedId: string;
      resolvedUrl: string;
      durationMs: number;
      timestamp: number;
    }
  | {
      type: 'import:start';
      traceId: string;
      request: string;
      resolvedId: string;
      timestamp: number;
    }
  | {
      type: 'import:end';
      traceId: string;
      request: string;
      moduleId: string;
      durationMs: number;
      timestamp: number;
    }
  | {
      type: 'import:error';
      traceId: string;
      request: string;
      resolvedId?: string;
      error: unknown;
      timestamp: number;
    }
  | {
      type: 'fetch:start';
      traceId: string;
      request: string;
      importer?: string;
      cached: boolean;
      timestamp: number;
    }
  | {
      type: 'fetch:end';
      traceId: string;
      request: string;
      importer?: string;
      fromCache: boolean;
      resultKind: 'cache' | 'external' | 'inlined';
      durationMs: number;
      timestamp: number;
    }
  | {
      type: 'fetch:error';
      traceId: string;
      request: string;
      importer?: string;
      cached: boolean;
      error: unknown;
      timestamp: number;
    }
  | {
      type: 'eval:start';
      traceId: string;
      moduleId: string;
      mode: 'external' | 'inlined';
      timestamp: number;
    }
  | {
      type: 'eval:end';
      traceId: string;
      moduleId: string;
      mode: 'external' | 'inlined';
      durationMs: number;
      timestamp: number;
    }
  | {
      type: 'eval:error';
      traceId: string;
      moduleId: string;
      mode: 'external' | 'inlined';
      error: unknown;
      timestamp: number;
    }
  | {
      type: 'external:policy';
      traceId: string;
      requestId: string;
      externalize: string;
      externalType: ExternalFetchResult['type'];
      policy: 'globals' | 'custom' | 'builtin' | 'network';
      action: 'resolved' | 'stub' | 'error' | 'externalize';
      timestamp: number;
    }
  | {
      type: 'hmr:update';
      updates: number;
      timestamp: number;
    }
  | {
      type: 'hmr:prune';
      paths: number;
      timestamp: number;
    }
  | {
      type: 'hmr:full-reload';
      path?: string;
      timestamp: number;
    }
  | {
      type: 'interop:wrap';
      traceId: string;
      specifier: string;
      importer: string;
      kind: 'esm-namespace' | 'cjs-value';
      timestamp: number;
    }
  | {
      type: 'interop:error';
      traceId: string;
      specifier: string;
      importer: string;
      code: string;
      message: string;
      timestamp: number;
    };

export interface ModuleRunnerDiagnosticsEmitter {
  emit(event: ModuleRunnerDiagnosticsEvent): void;
  subscribe(
    listener: (event: ModuleRunnerDiagnosticsEvent) => void
  ): () => void;
  clear(): void;
}

export interface FarmModuleRunnerOptions {
  transport: ModuleRunnerTransport;
  hmr?: boolean;
  /**
   * Fail fast when transform fallback happens (externalized with bailoutReason),
   * instead of silently continuing via externalize path.
   */
  strictTransform?: boolean;
  evaluator?: ModuleRunnerEvaluatorType | ModuleEvaluator;
  resolveExternalModule?: ExternalModuleResolver;
  sourceMapInterceptor?: boolean | RunnerSourceMapInterceptorOptions;
  createImportMeta?: (
    modulePath: string
  ) => FarmRunnerImportMeta | Promise<FarmRunnerImportMeta>;
  evaluatedModules?: EvaluatedModules;
  resolver?: RunnerResolver;
  nonJsPolicy?: NonJsPolicyOptions;
  externalPolicy?: RunnerExternalPolicyOptions;
  cachePolicy?: RunnerCachePolicyOptions;
  diagnostics?: ModuleRunnerDiagnosticsEmitter | false;
  /**
   * Internal escape hatch for convergence migration.
   * When true, runner skips built-in interop evaluator wrapping.
   */
  disableInterop?: boolean;
}

export type ResolvedFetchResult = (ExternalFetchResult | InlinedFetchResult) & {
  id: string;
  url: string;
};

export type ModuleRunnerInvokeHandlers = {
  [K in keyof InvokeMethods]: (
    ...args: Parameters<InvokeMethods[K]>
  ) => ReturnType<InvokeMethods[K]>;
};
