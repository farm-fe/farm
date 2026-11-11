import { createHash } from 'node:crypto';
import { readFileSync } from 'node:fs';
import { runInNewContext } from 'node:vm';
import ts from 'typescript';
import { describe, expect, test, vi } from 'vitest';

const template = readFileSync(
  new URL(
    '../../../crates/plugin_lazy_compilation/src/dynamic_module.ts',
    import.meta.url
  ),
  'utf8'
);
const prefix = (namespace: string) =>
  `${createHash('sha256').update(namespace).digest('hex').slice(0, 16)}:`;

function runtime(namespace: string, styleIds?: string[]) {
  const cache: Record<string, unknown> = {};
  const required: string[] = [];
  const cacheAtRequire: string[][] = [];
  const system = {
    // Avoid starting a network request while evaluating the actual template.
    compilingModules: new Map([['MODULE_PATH', Promise.resolve()]]),
    c: () => cache,
    r(id: string) {
      required.push(id);
      cacheAtRequire.push(Object.keys(cache));
      cache[id] = 'refreshed';
    }
  };
  const document =
    styleIds === undefined
      ? undefined
      : {
          querySelectorAll: vi.fn(() =>
            styleIds.map((id) => ({
              getAttribute: () => id
            }))
          )
        };
  const source = template
    .replace("'FARM_MODULE_SYSTEM'", 'globalThis.system')
    .replace('FARM_CSS_STYLE_ID_PREFIX', prefix(namespace));
  const compiled = ts.transpileModule(
    `${source}\nexport { refreshUpdatedStyles };`,
    {
      compilerOptions: {
        target: ts.ScriptTarget.ES2022,
        module: ts.ModuleKind.CommonJS
      }
    }
  ).outputText;
  const exports: { refreshUpdatedStyles?: (changed?: string[]) => void } = {};
  runInNewContext(compiled, { system, document, exports });
  const refresh = exports.refreshUpdatedStyles;
  if (!refresh) throw new Error('Lazy CSS refresh function was not exported');
  return {
    cache,
    required,
    cacheAtRequire,
    document,
    refresh
  };
}

describe('lazy compilation namespaced CSS refresh', () => {
  test('refreshes raw module IDs only in the runtime that owns a loaded style', () => {
    const styles = [
      `${prefix('app-a')}button.css`,
      `${prefix('app-b')}button.css`,
      `${prefix('app-b')}other.css`
    ];
    const a = runtime('app-a', styles);
    const b = runtime('app-b', styles);
    a.cache['button.css'] = b.cache['button.css'] = 'old';
    a.cache['other.css'] = b.cache['other.css'] = 'old';

    a.refresh(['button.css', 'other.css']);
    expect(a.required).toEqual(['button.css']);
    expect(a.cache['other.css']).toBe('old');
    expect(b.cache['button.css']).toBe('old');

    b.refresh(['button.css', 'other.css']);
    expect(b.required).toEqual(['button.css', 'other.css']);
  });

  test('refreshes the default runtime namespace', () => {
    const namespace = '__farm_default_namespace__';
    const current = runtime(namespace, [`${prefix(namespace)}styles.css`]);
    current.cache['styles.css'] = 'old';
    current.refresh(['styles.css']);
    expect(current.required).toEqual(['styles.css']);
  });

  test('deduplicates changes and invalidates the entire batch before requiring', () => {
    const current = runtime('app-a', [
      `${prefix('app-a')}a.css`,
      `${prefix('app-a')}b.css`
    ]);
    current.cache['a.css'] = current.cache['b.css'] = 'old';
    current.cache['unchanged.css'] = 'old';
    current.refresh(['a.css', 'b.css', 'a.css', 'missing.css']);
    expect(current.required).toEqual(['a.css', 'b.css']);
    expect(current.cacheAtRequire).toEqual([
      ['unchanged.css'],
      ['unchanged.css', 'a.css']
    ]);
  });

  test('does nothing without a document', () => {
    const current = runtime('app-a');
    current.cache['styles.css'] = 'old';
    current.refresh(['styles.css']);
    expect(current.required).toEqual([]);
    expect(current.cache['styles.css']).toBe('old');
  });

  test('does not query styles for missing or empty changes', () => {
    const current = runtime('app-a', [`${prefix('app-a')}styles.css`]);
    current.refresh();
    current.refresh([]);
    expect(current.document?.querySelectorAll).not.toHaveBeenCalled();
    expect(current.required).toEqual([]);
  });
});
