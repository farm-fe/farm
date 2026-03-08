import { cleanUrl } from '../utils/url.js';
import type { RunnerSourceMapHooks } from './types.js';

type StackTraceFormatter = (error: Error, trace: NodeJS.CallSite[]) => unknown;

type SourceMapEntry = {
  originalSource: string | null;
  originalLine: number | null;
  originalColumn: number | null;
};

type SourceMapLike = {
  findEntry(lineOffset: number, columnOffset: number): SourceMapEntry;
};

type NodeSourceMapConstructor = new (payload: unknown) => SourceMapLike;

export interface RunnerSourceMapInterceptor {
  register(sourceId: string, sourceMap: string): void;
  unregister(sourceId: string): void;
  clear(): void;
  close(): void;
}

type GlobalCandidateEntry = {
  storeId: number;
  map: SourceMapLike;
};

type HookStoreEntry = {
  hooks: RunnerSourceMapHooks;
  parsedCache: Map<string, SourceMapLike | null>;
};

const globalStores = new Map<number, Map<string, SourceMapLike>>();
const globalCandidateEntries = new Map<string, GlobalCandidateEntry[]>();
const globalHookStores = new Map<number, HookStoreEntry>();
const STACK_FRAME_PATTERN = /^(\s*at\s+(?:.+?\s+\()?)(.+):(\d+):(\d+)(\)?)$/;

let installedCount = 0;
let nativeRequestedCount = 0;
let previousPrepareStackTrace: StackTraceFormatter | undefined;
let previousNativeSourceMapEnabled: boolean | undefined;
let touchedNativeSourceMapSwitch = false;
let nextStoreId = 0;
let nodeSourceMapCtor: NodeSourceMapConstructor | undefined;

function getProcessLike(): Record<string, unknown> | undefined {
  const processLike = Reflect.get(globalThis as object, 'process');

  if (!processLike || typeof processLike !== 'object') {
    return undefined;
  }

  return processLike as Record<string, unknown>;
}

function resolveNodeSourceMapCtor(): NodeSourceMapConstructor | undefined {
  if (nodeSourceMapCtor) {
    return nodeSourceMapCtor;
  }
  const processLike = getProcessLike();

  if (!processLike) {
    return undefined;
  }

  const getBuiltinModule = Reflect.get(processLike, 'getBuiltinModule');

  if (typeof getBuiltinModule !== 'function') {
    return undefined;
  }

  for (const moduleName of ['node:module', 'module']) {
    try {
      const nodeModule = Reflect.apply(
        getBuiltinModule as (...args: unknown[]) => unknown,
        processLike,
        [moduleName]
      ) as Record<string, unknown> | undefined;
      const sourceMapCtor = nodeModule && Reflect.get(nodeModule, 'SourceMap');

      if (typeof sourceMapCtor === 'function') {
        nodeSourceMapCtor = sourceMapCtor as NodeSourceMapConstructor;
        return nodeSourceMapCtor;
      }
    } catch {
      // ignore and try next candidate
    }
  }

  return undefined;
}

function sourceCandidates(source: string): string[] {
  const normalized = cleanUrl(source);
  return normalized === source ? [source] : [source, normalized];
}

function parseSourceMap(raw: string): SourceMapLike | undefined {
  if (typeof raw !== 'string' || raw.length === 0) {
    return undefined;
  }

  const SourceMapCtor = resolveNodeSourceMapCtor();

  if (!SourceMapCtor) {
    return undefined;
  }

  try {
    const payload = JSON.parse(raw) as unknown;
    return new SourceMapCtor(payload);
  } catch {
    return undefined;
  }
}

function lookupSourceMap(source: string): SourceMapLike | undefined {
  const candidates = sourceCandidates(source);

  for (const candidate of candidates) {
    const entries = globalCandidateEntries.get(candidate);
    const mapped = entries?.[entries.length - 1]?.map;

    if (mapped) {
      return mapped;
    }
  }

  const hookEntries = [...globalHookStores.entries()].reverse();
  for (const [, entry] of hookEntries) {
    for (const candidate of candidates) {
      const resolved = resolveHookSourceCandidate(entry.hooks, candidate);
      for (const hookCandidate of sourceCandidates(resolved)) {
        const fromHook = resolveHookSourceMap(entry, hookCandidate);
        if (fromHook) {
          return fromHook;
        }
      }
    }
  }

  return undefined;
}

function resolveHookSourceCandidate(
  hooks: RunnerSourceMapHooks,
  source: string
): string {
  if (!hooks.retrieveFile) {
    return source;
  }

  try {
    const resolved = hooks.retrieveFile(source);
    if (typeof resolved === 'string' && resolved.length > 0) {
      return resolved;
    }
  } catch {
    // keep best-effort behavior
  }

  return source;
}

function resolveHookSourceMap(
  entry: HookStoreEntry,
  source: string
): SourceMapLike | undefined {
  if (!entry.hooks.retrieveSourceMap) {
    return undefined;
  }

  if (entry.parsedCache.has(source)) {
    return entry.parsedCache.get(source) ?? undefined;
  }

  let raw: string | null | undefined;
  try {
    raw = entry.hooks.retrieveSourceMap(source);
  } catch {
    entry.parsedCache.set(source, null);
    return undefined;
  }

  if (typeof raw !== 'string' || raw.length === 0) {
    entry.parsedCache.set(source, null);
    return undefined;
  }

  const parsed = parseSourceMap(raw) ?? null;
  entry.parsedCache.set(source, parsed);
  return parsed ?? undefined;
}

function upsertGlobalCandidate(
  storeId: number,
  candidate: string,
  map: SourceMapLike
): void {
  const entries = globalCandidateEntries.get(candidate) ?? [];
  const filtered = entries.filter((entry) => entry.storeId !== storeId);
  filtered.push({ storeId, map });
  globalCandidateEntries.set(candidate, filtered);
}

function removeGlobalCandidate(storeId: number, candidate: string): void {
  const entries = globalCandidateEntries.get(candidate);

  if (!entries) {
    return;
  }

  const filtered = entries.filter((entry) => entry.storeId !== storeId);

  if (filtered.length === 0) {
    globalCandidateEntries.delete(candidate);
    return;
  }

  globalCandidateEntries.set(candidate, filtered);
}

function remapStackFrame(line: string): string {
  const matched = line.match(STACK_FRAME_PATTERN);

  if (!matched) {
    return line;
  }

  const [, prefix, source, lineNumber, columnNumber, suffix] = matched;
  const sourceMap = lookupSourceMap(source);

  if (!sourceMap) {
    return line;
  }

  const lineOffset = Number(lineNumber) - 1;
  const columnOffset = Number(columnNumber) - 1;

  if (lineOffset < 0 || columnOffset < 0) {
    return line;
  }

  const entry = sourceMap.findEntry(lineOffset, columnOffset);

  if (!entry || !entry.originalSource) {
    return line;
  }

  const mappedLine = (entry.originalLine ?? 0) + 1;
  const mappedColumn = (entry.originalColumn ?? 0) + 1;
  return `${prefix}${entry.originalSource}:${mappedLine}:${mappedColumn}${suffix}`;
}

function remapStackTrace(stack: string): string {
  return stack
    .split('\n')
    .map((line, index) => (index === 0 ? line : remapStackFrame(line)))
    .join('\n');
}

function applyCustomStackFormatter(
  error: Error,
  remappedStack: string,
  trace: NodeJS.CallSite[]
): unknown {
  const hookEntries = [...globalHookStores.entries()].reverse();
  for (const [, entry] of hookEntries) {
    const formatter = entry.hooks.formatStack;
    if (!formatter) {
      continue;
    }

    try {
      const formatted = formatter({
        error,
        remappedStack,
        trace
      });
      if (formatted !== undefined) {
        return formatted;
      }
    } catch {
      // Keep default behavior when formatter throws.
    }
  }

  return remappedStack;
}

function defaultFormatStackTrace(
  error: Error,
  trace: NodeJS.CallSite[]
): string {
  const errorName = error.name || 'Error';
  const message = error.message || '';
  const header = message ? `${errorName}: ${message}` : errorName;

  if (!trace.length) {
    return header;
  }

  return `${header}\n${trace.map((callSite) => `    at ${callSite.toString()}`).join('\n')}`;
}

function tryEnableNativeSourceMaps(): void {
  const processLike = getProcessLike();

  if (!processLike) {
    return;
  }

  const toggler = Reflect.get(processLike, 'setSourceMapsEnabled');

  if (typeof toggler !== 'function') {
    return;
  }

  try {
    const previous = Reflect.get(processLike, 'sourceMapsEnabled');

    if (typeof previous === 'boolean') {
      previousNativeSourceMapEnabled = previous;
    }

    Reflect.apply(toggler as (enabled: boolean) => void, processLike, [true]);
    touchedNativeSourceMapSwitch = true;
  } catch {
    touchedNativeSourceMapSwitch = false;
  }
}

function restoreNativeSourceMaps(): void {
  if (!touchedNativeSourceMapSwitch) {
    return;
  }

  const processLike = getProcessLike();

  if (!processLike) {
    return;
  }

  const toggler = Reflect.get(processLike, 'setSourceMapsEnabled');

  if (typeof toggler !== 'function') {
    return;
  }

  if (typeof previousNativeSourceMapEnabled !== 'boolean') {
    return;
  }

  try {
    Reflect.apply(toggler as (enabled: boolean) => void, processLike, [
      previousNativeSourceMapEnabled
    ]);
  } catch {
    // best effort restore
  } finally {
    previousNativeSourceMapEnabled = undefined;
    touchedNativeSourceMapSwitch = false;
  }
}

function ensureInstalled(useNative: boolean): boolean {
  if (useNative) {
    nativeRequestedCount++;
  }

  if (installedCount === 0) {
    previousPrepareStackTrace = Error.prepareStackTrace as
      | StackTraceFormatter
      | undefined;
    if (useNative) {
      tryEnableNativeSourceMaps();
    }

    try {
      Error.prepareStackTrace = (
        error: Error,
        trace: NodeJS.CallSite[]
      ): unknown => {
        const prepared = previousPrepareStackTrace
          ? previousPrepareStackTrace(error, trace)
          : defaultFormatStackTrace(error, trace);

        if (typeof prepared !== 'string') {
          return prepared;
        }

        const remapped = remapStackTrace(prepared);
        return applyCustomStackFormatter(error, remapped, trace);
      };
    } catch {
      if (useNative && nativeRequestedCount > 0) {
        nativeRequestedCount--;
      }
      restoreNativeSourceMaps();
      previousPrepareStackTrace = undefined;
      return false;
    }
  } else if (useNative && nativeRequestedCount === 1) {
    // A native-enabled interceptor is created after non-native ones.
    tryEnableNativeSourceMaps();
  }

  installedCount++;
  return true;
}

function ensureUninstalled(useNative: boolean): void {
  if (useNative && nativeRequestedCount > 0) {
    nativeRequestedCount--;
  }

  if (installedCount <= 0) {
    return;
  }

  installedCount--;

  if (installedCount > 0) {
    if (nativeRequestedCount === 0) {
      restoreNativeSourceMaps();
    }
    return;
  }

  nativeRequestedCount = 0;

  try {
    Error.prepareStackTrace = previousPrepareStackTrace as
      | ErrorConstructor['prepareStackTrace']
      | undefined;
  } catch {
    // Host can lock Error.prepareStackTrace. Keep shutdown best-effort.
  } finally {
    previousPrepareStackTrace = undefined;
    restoreNativeSourceMaps();
  }
}

const noopInterceptor: RunnerSourceMapInterceptor = {
  register() {},
  unregister() {},
  clear() {},
  close() {}
};

export function createRunnerSourceMapInterceptor(
  enabled = true,
  useNative = true,
  hooks?: RunnerSourceMapHooks
): RunnerSourceMapInterceptor {
  if (!enabled) {
    return noopInterceptor;
  }

  if (!ensureInstalled(useNative)) {
    return noopInterceptor;
  }

  const storeId = ++nextStoreId;
  const store = new Map<string, SourceMapLike>();
  const hookEntry: HookStoreEntry | undefined = hooks
    ? {
        hooks,
        parsedCache: new Map<string, SourceMapLike | null>()
      }
    : undefined;
  let closed = false;

  globalStores.set(storeId, store);
  if (hookEntry) {
    globalHookStores.set(storeId, hookEntry);
  }

  return {
    register(sourceId: string, sourceMap: string): void {
      if (closed) {
        return;
      }

      const parsed = parseSourceMap(sourceMap);

      if (!parsed) {
        return;
      }

      for (const candidate of new Set(sourceCandidates(sourceId))) {
        store.set(candidate, parsed);
        upsertGlobalCandidate(storeId, candidate, parsed);
      }
    },
    unregister(sourceId: string): void {
      if (closed) {
        return;
      }

      for (const candidate of sourceCandidates(sourceId)) {
        if (store.delete(candidate)) {
          removeGlobalCandidate(storeId, candidate);
        }
      }
      hookEntry?.parsedCache.clear();
    },
    clear(): void {
      if (closed) {
        return;
      }

      for (const candidate of store.keys()) {
        removeGlobalCandidate(storeId, candidate);
      }
      store.clear();
      hookEntry?.parsedCache.clear();
    },
    close(): void {
      if (closed) {
        return;
      }

      closed = true;
      for (const candidate of store.keys()) {
        removeGlobalCandidate(storeId, candidate);
      }
      store.clear();
      globalStores.delete(storeId);
      globalHookStores.delete(storeId);
      ensureUninstalled(useNative);
    }
  };
}
