import { existsSync } from 'node:fs';
import fs from 'node:fs/promises';
import { builtinModules, createRequire } from 'node:module';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import type { Server } from '../server/index.js';
import { cleanUrl } from '../utils/url.js';
import type {
  FetchFunctionOptions,
  FetchResult,
  InlinedFetchResult,
  ModuleRunnerInvokeContext,
  ModuleRunnerInvokeHandlers,
  RunnerTransformBailoutReason
} from './types.js';

type RunnerInvokeState = {
  resourceMirrorStamp: number;
  resourceMirrorDir: string;
  moduleStamps: Map<string, number>;
  moduleFiles: Map<string, { path: string; mtimeMs: number }>;
  packageTypeByJsonPath: Map<string, 'module' | 'commonjs'>;
};

const BUILTINS = new Set([
  ...builtinModules,
  ...builtinModules.map((id) => `node:${id}`)
]);

const STATE = new WeakMap<ModuleRunnerInvokeContext, RunnerInvokeState>();

function getState(context: ModuleRunnerInvokeContext): RunnerInvokeState {
  const existing = STATE.get(context);

  if (existing) {
    return existing;
  }

  const created = {
    resourceMirrorStamp: -1,
    resourceMirrorDir: path.join(context.root, '.farm', 'module-runner'),
    moduleStamps: new Map<string, number>(),
    moduleFiles: new Map<string, { path: string; mtimeMs: number }>(),
    packageTypeByJsonPath: new Map<string, 'module' | 'commonjs'>()
  } satisfies RunnerInvokeState;

  STATE.set(context, created);

  return created;
}

function appendVersion(url: string, version: number | string): string {
  const mark = url.includes('?') ? '&' : '?';
  return `${url}${mark}t=${version}`;
}

async function readMtimeMs(filePath: string): Promise<number | null> {
  try {
    const stats = await fs.stat(filePath);
    return stats.mtimeMs;
  } catch {
    return null;
  }
}

async function getModuleTypeByFile(
  file: string,
  state: RunnerInvokeState
): Promise<'module' | 'commonjs'> {
  if (file.endsWith('.cjs')) {
    return 'commonjs';
  }

  if (file.endsWith('.mjs')) {
    return 'module';
  }

  if (!file.endsWith('.js')) {
    return 'module';
  }

  let current = path.dirname(file);

  while (true) {
    const packageJsonPath = path.join(current, 'package.json');
    const cached = state.packageTypeByJsonPath.get(packageJsonPath);

    if (cached) {
      return cached;
    }

    try {
      const raw = await fs.readFile(packageJsonPath, 'utf-8');
      const parsed = JSON.parse(raw) as { type?: unknown };
      const type = parsed.type === 'module' ? 'module' : 'commonjs';
      state.packageTypeByJsonPath.set(packageJsonPath, type);
      return type;
    } catch (error) {
      if ((error as { code?: string }).code !== 'ENOENT') {
        return 'commonjs';
      }
    }

    const parent = path.dirname(current);
    if (parent === current) {
      break;
    }
    current = parent;
  }

  return 'commonjs';
}

function resolveExternalizedFilePath(externalize: string): string | null {
  const normalized = cleanUrl(externalize);

  if (normalized.startsWith('file://')) {
    try {
      return fileURLToPath(normalized);
    } catch {
      return null;
    }
  }

  if (path.isAbsolute(normalized)) {
    return normalized;
  }

  return null;
}

async function normalizeExternalizedTypeByPath(
  state: RunnerInvokeState,
  externalize: string,
  type: 'module' | 'commonjs' | 'builtin' | 'network'
): Promise<'module' | 'commonjs' | 'builtin' | 'network'> {
  if (type === 'builtin' || type === 'network') {
    return type;
  }

  const filePath = resolveExternalizedFilePath(externalize);

  if (!filePath || !existsSync(filePath)) {
    return type;
  }

  return getModuleTypeByFile(filePath, state);
}

function isExternalType(
  value: unknown
): value is 'module' | 'commonjs' | 'builtin' | 'network' {
  return (
    value === 'module' ||
    value === 'commonjs' ||
    value === 'builtin' ||
    value === 'network'
  );
}

function isBailoutReason(
  value: unknown
): value is RunnerTransformBailoutReason {
  return (
    value === 'unsupported-ts' ||
    value === 'import-mutation' ||
    value === 'unhandled-module-decl' ||
    value === 'not-script' ||
    value === 'empty-content'
  );
}

function isFetchResult(value: unknown): value is FetchResult {
  if (!value || typeof value !== 'object') {
    return false;
  }

  if ('cache' in value) {
    return (value as { cache?: unknown }).cache === true;
  }

  if ('externalize' in value) {
    const externalized = value as {
      externalize?: unknown;
      type?: unknown;
      bailoutReason?: unknown;
    };

    return (
      typeof externalized.externalize === 'string' &&
      isExternalType(externalized.type) &&
      (externalized.bailoutReason == null ||
        isBailoutReason(externalized.bailoutReason))
    );
  }

  if ('code' in value) {
    const inlined = value as {
      code?: unknown;
      file?: unknown;
      id?: unknown;
      url?: unknown;
      invalidate?: unknown;
      map?: unknown;
    };

    return (
      typeof inlined.code === 'string' &&
      (inlined.file == null || typeof inlined.file === 'string') &&
      typeof inlined.id === 'string' &&
      typeof inlined.url === 'string' &&
      typeof inlined.invalidate === 'boolean' &&
      (inlined.map == null || typeof inlined.map === 'string')
    );
  }

  return false;
}

function normalizeInlinedFetchResult(
  result: InlinedFetchResult
): InlinedFetchResult {
  return {
    ...result,
    map: result.map ?? null
  };
}

async function mirrorCompiledResources(
  context: ModuleRunnerInvokeContext
): Promise<string> {
  const state = getState(context);
  const compiler = context.compiler;

  if (!compiler) {
    throw new Error('[farm module runner] compiler is not initialized.');
  }

  if (state.resourceMirrorStamp === context.moduleRunnerStamp) {
    return state.resourceMirrorDir;
  }

  const resources = compiler.resources();

  await fs.rm(state.resourceMirrorDir, { recursive: true, force: true });
  await fs.mkdir(state.resourceMirrorDir, { recursive: true });

  await Promise.all(
    Object.entries(resources).map(async ([name, content]) => {
      const file = path.join(state.resourceMirrorDir, name);
      await fs.mkdir(path.dirname(file), { recursive: true });
      await fs.writeFile(file, content);
    })
  );

  state.resourceMirrorStamp = context.moduleRunnerStamp;

  return state.resourceMirrorDir;
}

function normalizeImporter(
  importer: string | undefined,
  root: string
): string | undefined {
  if (!importer) {
    return undefined;
  }

  const normalized = cleanUrl(importer);

  if (normalized.startsWith('file://')) {
    return fileURLToPath(normalized);
  }

  if (normalized.startsWith('/')) {
    return path.resolve(root, normalized.slice(1));
  }

  const rooted = path.resolve(root, normalized);
  if (existsSync(rooted)) {
    return rooted;
  }

  return normalized;
}

function resolveResourceNameByUrl(url: string, publicPath: string): string {
  let normalized = cleanUrl(url);

  if (publicPath !== '/' && normalized.startsWith(publicPath)) {
    normalized = normalized.slice(publicPath.length);
  }

  if (normalized.startsWith('/')) {
    normalized = normalized.slice(1);
  }

  return normalized;
}

function resolveFilePath(
  context: ModuleRunnerInvokeContext,
  id: string,
  importer?: string
): string | undefined {
  const normalized = cleanUrl(id);

  if (normalized.startsWith('file://')) {
    return fileURLToPath(normalized);
  }

  if (normalized.startsWith('/')) {
    return path.resolve(context.root, normalized.slice(1));
  }

  if (normalized.startsWith('.')) {
    const importerPath = normalizeImporter(importer, context.root);

    if (!importerPath) {
      return path.resolve(context.root, normalized);
    }

    return path.resolve(path.dirname(importerPath), normalized);
  }

  return undefined;
}

function resolveViteVirtualFetchResult(id: string): InlinedFetchResult | null {
  if (id !== '\0plugin-vue:export-helper') {
    return null;
  }

  return {
    code: [
      'function exportHelper(sfc, props) {',
      '  const target = sfc.__vccOpts || sfc;',
      '  for (let i = 0; i < props.length; i++) {',
      '    const pair = props[i];',
      '    target[pair[0]] = pair[1];',
      '  }',
      '  return target;',
      '}',
      '__farm_ssr_export_name__("default", () => exportHelper);'
    ].join('\n'),
    file: null,
    id,
    url: id,
    invalidate: false,
    map: null
  };
}

function isVueStyleQueryRequest(id: string): boolean {
  const cleaned = cleanUrl(id);
  return (
    cleaned.endsWith('.vue') &&
    /(?:\?|&)vue(?:&|$)/.test(id) &&
    /(?:\?|&)type=style(?:&|$)/.test(id)
  );
}

function resolveVueStyleNoopFetchResult(
  context: ModuleRunnerInvokeContext,
  id: string,
  importer: string | undefined,
  bailoutReason: RunnerTransformBailoutReason | undefined
): InlinedFetchResult | null {
  if (bailoutReason !== 'not-script' || !isVueStyleQueryRequest(id)) {
    return null;
  }

  const file = resolveFilePath(context, id, importer) ?? null;

  return {
    code: [
      'const __farm_ssr_style_noop__ = {};',
      '__farm_ssr_export_name__("default", () => __farm_ssr_style_noop__);'
    ].join('\n'),
    file,
    id,
    url: id,
    invalidate: false,
    map: null
  };
}

function resolveRootRelativeCompilerFetchId(
  context: ModuleRunnerInvokeContext,
  id: string
): string | null {
  const normalized = cleanUrl(id);

  if (!normalized.startsWith('/')) {
    return null;
  }

  if (existsSync(normalized)) {
    return null;
  }

  const resourceName = resolveResourceNameByUrl(normalized, context.publicPath);
  if (!resourceName) {
    return null;
  }

  const suffix = id.slice(normalized.length);
  return `${path.resolve(context.root, resourceName)}${suffix}`;
}

function fetchModuleFromCompiler(
  context: ModuleRunnerInvokeContext,
  id: string,
  importer: string | undefined,
  options: FetchFunctionOptions | undefined
): unknown {
  const compiler = context.compiler;
  if (!compiler) {
    return null;
  }

  const candidates = [id];
  const rootRelativeId = resolveRootRelativeCompilerFetchId(context, id);
  if (rootRelativeId && rootRelativeId !== id) {
    candidates.push(rootRelativeId);
  }

  for (const candidate of candidates) {
    const result = compiler.fetchModule(candidate, importer, options);
    if (result != null) {
      return result;
    }
  }

  return null;
}

async function resolveFetchModule(
  context: ModuleRunnerInvokeContext,
  id: string,
  importer?: string,
  options?: FetchFunctionOptions
): Promise<FetchResult> {
  const compiler = context.compiler;

  if (!compiler) {
    throw new Error(
      '[farm module runner] compiler is not initialized, call createModuleRunner after server compile.'
    );
  }

  if (compiler.compiling) {
    await compiler.waitForCompileFinish();
  }

  const state = getState(context);
  const cacheKey = `${importer ?? '<entry>'}::${id}`;
  const latestStamp = context.moduleRunnerStamp;

  if (options?.cached && state.moduleStamps.get(cacheKey) === latestStamp) {
    const cachedFile = state.moduleFiles.get(cacheKey);

    if (!cachedFile) {
      return { cache: true };
    }

    const mtimeMs = await readMtimeMs(cachedFile.path);

    if (mtimeMs != null && mtimeMs === cachedFile.mtimeMs) {
      return { cache: true };
    }

    state.moduleFiles.delete(cacheKey);
  }

  const compilerFetchResult = fetchModuleFromCompiler(
    context,
    id,
    importer,
    options
  );

  if (compilerFetchResult != null) {
    if (!isFetchResult(compilerFetchResult)) {
      throw new Error(
        `[farm module runner] Invalid fetchModule result for "${id}".`
      );
    }

    if (!('cache' in compilerFetchResult)) {
      state.moduleStamps.set(cacheKey, latestStamp);
      state.moduleFiles.delete(cacheKey);
    }

    if ('code' in compilerFetchResult) {
      return normalizeInlinedFetchResult(compilerFetchResult);
    }

    if ('externalize' in compilerFetchResult) {
      const vueStyleNoopResult = resolveVueStyleNoopFetchResult(
        context,
        id,
        importer,
        compilerFetchResult.bailoutReason
      );
      if (vueStyleNoopResult) {
        return vueStyleNoopResult;
      }

      return {
        ...compilerFetchResult,
        type: await normalizeExternalizedTypeByPath(
          state,
          compilerFetchResult.externalize,
          compilerFetchResult.type
        )
      };
    }

    return compilerFetchResult;
  }

  const normalized = cleanUrl(id);

  if (normalized.startsWith('data:')) {
    state.moduleFiles.delete(cacheKey);
    return {
      externalize: normalized,
      type: 'builtin'
    };
  }

  if (BUILTINS.has(normalized)) {
    state.moduleFiles.delete(cacheKey);
    return {
      externalize: normalized,
      type: 'builtin'
    };
  }

  if (/^https?:\/\//.test(normalized)) {
    state.moduleFiles.delete(cacheKey);
    return {
      externalize: normalized,
      type: 'network'
    };
  }

  const viteVirtualResult = resolveViteVirtualFetchResult(normalized);
  if (viteVirtualResult) {
    state.moduleStamps.set(cacheKey, latestStamp);
    state.moduleFiles.delete(cacheKey);
    return viteVirtualResult;
  }

  const resourceName = resolveResourceNameByUrl(normalized, context.publicPath);

  if (resourceName && compiler.resource(resourceName)) {
    const mirrorDir = await mirrorCompiledResources(context);
    const filePath = path.join(mirrorDir, resourceName);
    const externalize = appendVersion(
      pathToFileURL(filePath).toString(),
      latestStamp
    );

    state.moduleStamps.set(cacheKey, latestStamp);
    state.moduleFiles.delete(cacheKey);

    return {
      externalize,
      type: await getModuleTypeByFile(filePath, state)
    };
  }

  const directFilePath = resolveFilePath(context, normalized, importer);

  if (directFilePath && existsSync(directFilePath)) {
    const mtimeMs = await readMtimeMs(directFilePath);

    const url = pathToFileURL(directFilePath).toString();
    const shouldVersion = directFilePath.startsWith(context.root);
    const version =
      mtimeMs == null ? latestStamp : `${latestStamp}-${Math.floor(mtimeMs)}`;
    const externalize = shouldVersion ? appendVersion(url, version) : url;

    state.moduleStamps.set(cacheKey, latestStamp);
    if (mtimeMs != null) {
      state.moduleFiles.set(cacheKey, { path: directFilePath, mtimeMs });
    } else {
      state.moduleFiles.delete(cacheKey);
    }

    return {
      externalize,
      type: await getModuleTypeByFile(directFilePath, state)
    };
  }

  const importerFile = normalizeImporter(importer, context.root);
  const resolveFrom =
    importerFile && existsSync(importerFile)
      ? importerFile
      : path.join(context.root, 'package.json');

  try {
    const require = createRequire(resolveFrom);
    const resolved = require.resolve(normalized);
    const filePath = cleanUrl(resolved);
    const mtimeMs = await readMtimeMs(filePath);
    const shouldVersion = filePath.startsWith(context.root);
    const version =
      mtimeMs == null ? latestStamp : `${latestStamp}-${Math.floor(mtimeMs)}`;
    const externalize = shouldVersion
      ? appendVersion(pathToFileURL(filePath).toString(), version)
      : pathToFileURL(filePath).toString();

    state.moduleStamps.set(cacheKey, latestStamp);
    if (mtimeMs != null) {
      state.moduleFiles.set(cacheKey, { path: filePath, mtimeMs });
    } else {
      state.moduleFiles.delete(cacheKey);
    }

    return {
      externalize,
      type: await getModuleTypeByFile(filePath, state)
    };
  } catch {
    throw new Error(
      `[farm module runner] Cannot fetch module "${id}"${
        importer ? ` imported from "${importer}"` : ''
      }. Fallback path supports built outputs, local files and externalized dependencies.`
    );
  }
}

export function createModuleRunnerInvokeHandlers(
  context: ModuleRunnerInvokeContext
): ModuleRunnerInvokeHandlers {
  return {
    fetchModule: (id, importer, options) =>
      resolveFetchModule(context, id, importer, options),
    getBuiltins: async () => Array.from(BUILTINS.values())
  };
}

export function createServerModuleRunnerInvokeHandlers(
  server: Server
): ModuleRunnerInvokeHandlers {
  const context = {
    get root() {
      return server.root;
    },
    get publicPath() {
      return server.publicPath;
    },
    get moduleRunnerStamp() {
      return server.moduleRunnerStamp;
    },
    get compiler() {
      return server.compiler;
    }
  } as ModuleRunnerInvokeContext;

  return createModuleRunnerInvokeHandlers(context);
}
