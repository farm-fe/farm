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
