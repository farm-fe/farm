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
import { EvaluatedModuleNode, EvaluatedModules } from './evaluatedModules.js';
import { createModuleEvaluator } from './evaluator.js';
import { createRunnerSourceMapInterceptor } from './sourceMapInterceptor.js';
import type {
  FarmModuleRunnerOptions,
  FetchResult,
  ModuleEvaluator,
  ResolvedFetchResult,
  RunnerHotPayload
} from './types.js';

export class FarmModuleRunner {
  readonly evaluatedModules: EvaluatedModules;
  private readonly sourceMapInterceptor: ReturnType<
    typeof createRunnerSourceMapInterceptor
  >;
  private readonly evaluator: ModuleEvaluator;
  private readonly reportedBailouts = new Set<string>();
  private readonly strictTransform: boolean;

  private closed = false;

  constructor(private readonly options: FarmModuleRunnerOptions) {
    this.strictTransform = options.strictTransform === true;

    this.evaluator = createModuleEvaluator(
      options.evaluator,
      options.resolveExternalModule
    );

    this.sourceMapInterceptor = createRunnerSourceMapInterceptor(
      this.resolveSourceMapInterceptorEnabled(options.sourceMapInterceptor),
      this.resolveSourceMapInterceptorNative(options.sourceMapInterceptor)
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
    const mod = await this.cachedModule(url);
    return (await this.cachedRequest(url, mod)) as T;
  }

  clearCache(): void {
    this.sourceMapInterceptor.clear();
    this.evaluatedModules.clear();
    this.reportedBailouts.clear();
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
    const fetched = (await this.options.transport.invoke('fetchModule', [
      url,
      importer,
      {
        cached: Boolean(cached),
        startOffset: this.evaluator.startOffset
      }
    ])) as FetchResult;

    if ('cache' in fetched) {
      if (!cached?.meta) {
        throw new Error(
          `[farm module runner] Module "${url}" returned cache=true before first load.`
        );
      }

      return cached;
    }

    const moduleId =
      'externalize' in fetched ? fetched.externalize : fetched.id;
    const moduleUrl = 'url' in fetched ? fetched.url : url;
    const mod = this.evaluatedModules.ensureModule(moduleId, moduleUrl);

    if ('invalidate' in fetched && fetched.invalidate) {
      this.evaluatedModules.invalidateModule(mod);
    }

    const normalized: ResolvedFetchResult = {
      ...fetched,
      id: moduleId,
      url: moduleUrl
    };

    mod.meta = normalized;

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

      if (dep[0] === '.') {
        dep = resolveRelativeRequestUrl(requestBase, dep);
      }

      const importer = meta.id;
      const depMod = await this.cachedModule(dep, importer);
      depMod.importers.add(importer);
      mod.imports.add(depMod.id);

      return this.cachedRequest(
        dep,
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
      const exports = await this.evaluator.runExternalModule(
        meta.externalize,
        meta.type,
        importOptions
      );
      mod.exports = exports;
      return exports;
    }

    if (!meta.code) {
      throw new Error(`[farm module runner] Missing code for "${url}".`);
    }

    const createImportMeta =
      this.options.createImportMeta ?? createDefaultImportMeta;
    const modulePath = meta.file || meta.id;
    const importMeta = await createImportMeta(modulePath);
    const exports: Record<string, unknown> = Object.create(null);

    Object.defineProperty(exports, Symbol.toStringTag, {
      value: 'Module',
      enumerable: false,
      configurable: false
    });

    mod.exports = exports;

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

    return exports;
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
      for (const update of payload.updates) {
        this.invalidateByUrl(update.path);
        this.invalidateByUrl(update.acceptedPath);
      }
      return;
    }

    if (payload.type === 'full-reload') {
      this.clearCache();
      return;
    }

    if (payload.type === 'prune') {
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

      this.evaluatedModules.invalidateModule(mod);

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
