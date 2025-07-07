import * as querystring from 'node:querystring';
import fse from 'fs-extra';
import type {
  InternalModuleFormat,
  NormalizedInputOptions,
  NormalizedOutputOptions,
  OutputAsset,
  OutputChunk,
  RenderedChunk
} from 'rollup';

import { VITE_ADAPTER_VIRTUAL_MODULE } from './constants.js';

import { readFile } from 'node:fs/promises';
import { ModuleContext, ModuleNode } from '../../config/types.js';
import type { Config } from '../../types/binding.js';
import { normalizePath } from '../../utils/share.js';
import type { JsPlugin, JsResourcePot, Resource } from '../type.js';
import { createModuleGraph } from './vite-server-adapter.js';

export type WatchChangeEvents = 'create' | 'update' | 'delete';

export function convertEnforceToPriority(value: 'pre' | 'post' | undefined) {
  const defaultPriority = 100;
  const enforceToPriority = {
    pre: 101,
    post: 98
  };

  if (value === undefined) {
    return defaultPriority;
  }

  return enforceToPriority[value] !== undefined
    ? enforceToPriority[value]
    : defaultPriority;
}

export function convertWatchEventChange(
  value: WatchChangeEvents
): WatchChangeEvents {
  const watchEventChange = {
    Added: 'create',
    Updated: 'update',
    Removed: 'delete'
  } as unknown as { [key in WatchChangeEvents]: WatchChangeEvents };

  return watchEventChange[value];
}

export function getContentValue(content: any): string {
  return encodeStr(typeof content === 'string' ? content : content.code);
}

export function customParseQueryString(url: string | null) {
  if (!url) {
    return [];
  }

  const queryString = url.split('?')[1];

  const parsedParams = querystring.parse(queryString);
  const paramsArray = [];

  for (const key in parsedParams) {
    paramsArray.push([key, parsedParams[key] as string]);
  }

  return paramsArray as [string, string][];
}

export const VITE_PLUGIN_DEFAULT_MODULE_TYPE =
  'VITE_PLUGIN_DEFAULT_MODULE_TYPE';

export const CSS_LANGS_RES: [RegExp, string][] = [
  [/\.(less)(?:$|\?)/, 'less'],
  [/\.(scss|sass)(?:$|\?)/, 'sass'],
  [/\.(styl|stylus)(?:$|\?)/, 'stylus'],
  [/\.(css)(?:$|\?)/, 'css']
];

export const JS_LANGS_RES: [RegExp, string][] = [
  [/\.(js|mjs|cjs|)(?:$|\?)/, 'js'],
  // jsx
  [/\.(jsx)(?:$|\?)/, 'jsx'],
  // ts
  [/\.(ts|cts|mts)(?:$|\?)/, 'ts'],
  // tsx
  [/\.(tsx)(?:$|\?)/, 'tsx']
];

export const DEFAULT_FILTERS = ['!node_modules'];
// farm css modules query
export const FARM_CSS_MODULES_SUFFIX = /(?:\?|&)farm_css_modules/;

export const stringifyQuery = (query: [string, string][]) => {
  if (!query.length) {
    return '';
  }

  let queryStr = '';

  for (const [key, value] of query) {
    queryStr += `${key}${value ? `=${value}` : ''}&`;
  }

  return `${queryStr.slice(0, -1)}`;
};

export function formatId(id: string, query: [string, string][]): string {
  // remove the adapter internal virtual module flag
  if (isStartAdapterVirtualModule(id)) {
    id = id?.replace(VITE_ADAPTER_VIRTUAL_MODULE, '');
  }

  if (!query.length) {
    return id;
  }

  return `${id}?${stringifyQuery(query)}`;
}

// determine if it is the adapter's internal virtual module
export function isStartAdapterVirtualModule(id: string) {
  return id?.startsWith(VITE_ADAPTER_VIRTUAL_MODULE);
}

export function isStartsWithSlash(str: string) {
  return str?.startsWith('/');
}

export function addAdapterVirtualModuleFlag(id: string) {
  return VITE_ADAPTER_VIRTUAL_MODULE + id;
}

export function normalizeAdapterVirtualModule(id: string) {
  const path = removeQuery(id);
  // If resolveIdResult is a path starting with / and the file at that path does not exist
  // then it is considered an internal virtual module
  if (isStartsWithSlash(path) && !fse.pathExistsSync(path))
    return addAdapterVirtualModuleFlag(id);
  return id;
}

export const removeQuery = (path: string) => {
  const queryIndex = path.indexOf('?');
  if (queryIndex !== -1) {
    return path.slice(0, queryIndex);
  }
  return revertNormalizePath(path.concat(''));
};

export function revertNormalizePath(p: string): string {
  return process.platform === 'win32' ? p.replace(/\//g, '\\') : p;
}

export function getCssModuleType(id: string): string | null {
  for (const [reg, lang] of CSS_LANGS_RES) {
    if (reg.test(id)) {
      return lang;
    }
  }

  return null;
}

export function getJsModuleType(id: string): string | null {
  for (const [reg, lang] of JS_LANGS_RES) {
    if (reg.test(id)) {
      return lang;
    }
  }

  return null;
}

export function formatLoadModuleType(id: string): string {
  const cssModuleType = getCssModuleType(id);

  if (cssModuleType) {
    return cssModuleType;
  }

  const jsModuleType = getJsModuleType(id);

  if (jsModuleType) {
    return jsModuleType;
  }

  return 'js';
}

export function formatTransformModuleType(id: string): string {
  return formatLoadModuleType(id);
}

// normalize invalid characters in id, for example: \0
// because characters like \0 have issues when passing to Farm's rust compiler
export function encodeStr(str: string): string {
  return str.replace(/\0/g, '\\0');
}

export function decodeStr(str: string): string {
  return str.replace(/\\0/g, '\0');
}

export function deleteUndefinedPropertyDeeply(obj: any) {
  if (typeof obj !== 'object') {
    return;
  }

  for (const key in obj) {
    if (!Object.prototype.hasOwnProperty.call(obj, key)) {
      continue;
    }

    if (Array.isArray(obj[key])) {
      obj[key] = obj[key].filter((item: any) => item !== undefined);
    } else if (obj[key] === undefined) {
      delete obj[key];
    } else if (typeof obj[key] === 'object') {
      deleteUndefinedPropertyDeeply(obj[key]);
    }
  }
}

export function throwIncompatibleError(
  pluginName: string,
  readingObject: string,
  allowedKeys: string[],
  key: string | number | symbol
): never {
  throw new Error(
    `Vite plugin '${pluginName}' is not compatible with Farm for now. Because it uses ${readingObject}['${String(
      key
    )}'] which is not supported by Farm. Current supported keys are: ${allowedKeys.join(
      ','
    )}`
  );
}

export function transformResourceInfo2RollupRenderedChunk(
  info: Partial<JsResourcePot>
): RenderedChunk {
  const { moduleIds, name, isEntry, isDynamicEntry } = info;

  return {
    dynamicImports: [],
    fileName: name,
    implicitlyLoadedBefore: [],
    importedBindings: {},
    imports: [],
    modules: {}, // do not support modules
    referencedFiles: [],
    exports: [],
    facadeModuleId: null,
    isDynamicEntry,
    isEntry,
    isImplicitEntry: false,
    moduleIds,
    name,
    type: 'chunk'
  } satisfies RenderedChunk;
}

export function transformResourceInfo2RollupResource(
  resource: Resource
): OutputChunk | OutputAsset {
  // Rollup/Vite only treat js files as chunk
  if (resource.resourceType === 'js') {
    const source = Buffer.from(resource.bytes).toString('utf-8');
    return {
      ...transformResourceInfo2RollupRenderedChunk({}),
      type: 'chunk',
      code: source,
      name: resource.name,
      map: undefined,
      sourcemapFileName: null,
      preliminaryFileName: resource.origin.value
    } satisfies OutputChunk;
  } else {
    return {
      fileName: resource.name,
      name: resource.name,
      needsCodeReference: false,
      source: Uint8Array.from(resource.bytes),
      type: 'asset'
    } satisfies OutputAsset;
  }
}

export function transformRollupResource2FarmResource(
  chunk: OutputChunk | OutputAsset,
  originResource: Resource
): Resource {
  if (chunk.type === 'chunk') {
    return {
      ...originResource,
      bytes: Array.from(Buffer.from(chunk.code)) as number[],
      emitted: originResource.emitted,
      name: chunk.name
    };
  } else {
    return {
      bytes: Array.from(chunk.source as Uint8Array) as number[],
      emitted: originResource.emitted,
      name: chunk.name,
      origin: originResource.origin,
      resourceType: originResource.resourceType
    };
  }
}

const notSupport: (method: string) => (...args: any[]) => any =
  (method) => () => {
    console.warn(`${method} not support`);
  };

const noop: (...args: any) => any = () => void 0;

function transformFarmFormatToRollupFormat(
  config: Config['config']['output']
): InternalModuleFormat {
  if (config.format === 'esm') {
    return 'es';
  } else if (config.format === 'cjs') {
    if (config.targetEnv === 'node') return 'cjs';
    return 'amd';
  }
}

export function transformFarmConfigToRollupNormalizedOutputOptions(
  config: Config['config']
): NormalizedOutputOptions {
  return {
    amd: { autoId: false, define: 'define', forceJsExtensionForImports: false },
    assetFileNames: config.output.assetsFilename,
    chunkFileNames: config.output.filename,
    compact: Boolean(config.minify),
    dir: config.output.path,
    dynamicImportInCjs: true,
    entryFileNames: config.output.entryFilename,
    esModule: 'if-default-prop',
    experimentalMinChunkSize: config?.partialBundling?.targetMinSize || 1,
    exports: 'auto',
    extend: false,
    externalImportAssertions: false,
    // externalImportAttributes: true,
    externalLiveBindings: true,
    format: transformFarmFormatToRollupFormat(config.output),
    freeze: false,
    generatedCode: {
      arrowFunctions: false,
      constBindings: false,
      objectShorthand: false,
      reservedNamesAsProps: true,
      symbols: false
    },

    globals: {},
    hoistTransitiveImports: true,
    indent: true,
    inlineDynamicImports: false,
    manualChunks: {},
    minifyInternalExports: true,
    noConflict: false,
    paths: {},
    plugins: [],
    preserveModules: false,
    sourcemap: Boolean(config.sourcemap),
    sourcemapExcludeSources: false,
    strict: true,
    systemNullSetters: true,
    validate: false,
    banner: notSupport('banner'),
    footer: notSupport('footer'),
    interop: notSupport('interop'),
    outro: notSupport('outro'),
    intro: notSupport('intro'),
    sanitizeFileName: notSupport('sanitizeFileName'),
    sourcemapIgnoreList: notSupport('sourcemapIgnoreList'),

    dynamicImportFunction: undefined,
    experimentalDeepDynamicChunkOptimization: false,
    file: undefined,
    name: undefined,
    namespaceToStringTag: false,
    preferConst: false,
    preserveModulesRoot: undefined,
    sourcemapBaseUrl: undefined,
    sourcemapFile: undefined,
    sourcemapFileNames: undefined,
    sourcemapPathTransform: undefined
  } satisfies NormalizedOutputOptions;
}

export function transformFarmConfigToRollupNormalizedInputOptions(
  config: Config['config']
): NormalizedInputOptions {
  return {
    context: 'undefined',
    experimentalCacheExpiry: 10,
    experimentalLogSideEffects: false,
    input: config.input,
    logLevel: 'info',
    makeAbsoluteExternalsRelative: 'ifRelativeSource',
    maxParallelFileOps: 20,
    perf: false,
    plugins: [],
    preserveEntrySignatures: 'exports-only',
    preserveSymlinks: false,
    shimMissingExports: false,
    strictDeprecations: false,
    treeshake: config.treeShaking && {
      moduleSideEffects: () => false,
      annotations: true,
      correctVarValueBeforeDeclaration: false,
      manualPureFunctions: [],
      propertyReadSideEffects: true,
      tryCatchDeoptimization: true,
      unknownGlobalSideEffects: true
    },
    acorn: undefined,
    acornInjectPlugins: undefined,
    cache: undefined,
    external: undefined,
    inlineDynamicImports: undefined,
    manualChunks: undefined,
    maxParallelFileReads: undefined,
    moduleContext: undefined,
    onLog: noop,
    onwarn: noop,
    preserveModules: undefined
  } satisfies NormalizedInputOptions;
}

export function normalizeFilterPath(path: string): string {
  if (process.platform === 'win32') {
    return compatibleWin32Path(path);
  }

  return path;
}

export function compatibleWin32Path(path: string): string {
  return path.replaceAll('/', '\\\\');
}

export function wrapPluginUpdateModules(plugin: JsPlugin): JsPlugin {
  if (!plugin.updateModules?.executor) {
    return plugin;
  }
  const originalExecutor = plugin.updateModules.executor;
  const moduleGraph = createModuleGraph(plugin.name);

  plugin.updateModules.executor = async ({ paths }, ctx) => {
    moduleGraph.context = ctx;
    // TODO order with sort by updateModules hooks priority
    for (const [file, type] of paths) {
      const mods = moduleGraph.getModulesByFile(
        file
      ) as unknown as ModuleNode[];

      const filename = normalizePath(file);
      const moduleContext: ModuleContext = {
        file: filename,
        timestamp: Date.now(),
        type,
        paths,
        modules: (mods ?? []).map(
          (m) =>
            ({
              ...m,
              id: normalizePath(m.id),
              file: normalizePath(m.file)
            }) as ModuleNode
        ),
        read: function (): string | Promise<string> {
          return readFile(file, 'utf-8');
        }
      };

      return originalExecutor.call(plugin, moduleContext);
    }
  };
  return plugin;
}
