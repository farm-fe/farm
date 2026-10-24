import { createHash } from 'node:crypto';
import { readFile, realpath } from 'node:fs/promises';
import { createRequire } from 'node:module';
import {
  Compiler as BindingCompiler,
  DocsSnapshot as BindingDocsSnapshot,
  bindingPath
} from '../binding/index.js';
import type { ResolvedUserConfig } from './config/types.js';

async function verifiedCompilerPath(): Promise<string> {
  const require = createRequire(import.meta.url);
  const loaded = Object.values(require.cache).filter(
    (entry) =>
      entry?.filename.endsWith('.node') &&
      entry.exports?.Compiler === BindingCompiler
  );
  const paths = new Set(
    await Promise.all(
      loaded.flatMap((entry) => (entry ? [realpath(entry.filename)] : []))
    )
  );
  const expected = await realpath(bindingPath);
  if (paths.size !== 1 || !paths.has(expected))
    throw new Error(
      '[DOCS_NATIVE_HOST] Loaded Core binary and plugin binding path differ; native overrides are unsupported for Docs'
    );
  return expected;
}

/** Identity of the binary actually resolved by the binding loader. */
export async function compilerNativeIdentity(): Promise<{
  path: string;
  sha256: string;
}> {
  if (!bindingPath) {
    throw new Error('[DOCS_NATIVE_HOST] Native binding path is unavailable');
  }
  const actual = await verifiedCompilerPath();
  const bytes = await readFile(actual);
  return {
    path: actual,
    sha256: createHash('sha256').update(bytes).digest('hex')
  };
}

/** Opaque Env-local native ownership; the handle must not be serialized. */
export type NativeDocsSnapshot = InstanceType<typeof BindingDocsSnapshot>;

export async function compileDocsTarget(
  resolved: ResolvedUserConfig,
  snapshot: NativeDocsSnapshot
): Promise<{
  resources: Record<string, Buffer>;
  inputObservations: unknown[];
}> {
  const result = await snapshot.compileTarget({
    config: resolved.compilation,
    jsPlugins: resolved.jsPlugins,
    rustPlugins: resolved.rustPlugins,
    docsSnapshot: snapshot
  });
  return {
    resources: result.resources,
    inputObservations: JSON.parse(result.inputObservations) as unknown[]
  };
}

/**
 * Prepare a private Docs snapshot without a synthetic compiler invocation.
 * The caller owns the output transaction and must await every operation before
 * removing its staging directory. Missing native support is an explicit error.
 */
export async function prepareDocsSnapshot(
  site: unknown,
  root: string,
  planPath: string
): Promise<NativeDocsSnapshot> {
  if (typeof BindingDocsSnapshot !== 'function') {
    throw new Error(
      '[DOCS_NATIVE_HOST] Rebuild the matching Core native binding; DocsSnapshot is unavailable'
    );
  }
  await verifiedCompilerPath();
  const snapshot = new BindingDocsSnapshot(root, JSON.stringify(site));
  try {
    await snapshot.prepare(planPath);
    return snapshot;
  } catch (error) {
    snapshot.close();
    throw error;
  }
}
