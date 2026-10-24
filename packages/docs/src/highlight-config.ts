import type { DocsSnippetsOptions } from '@farmfe/plugin-docs-snippets';
import type { CodeThemeConfig, SiteConfig } from './config.generated.js';

type Fields = Record<string, unknown>;
function object(value: unknown, label: string): Fields {
  if (!value || typeof value !== 'object' || Array.isArray(value))
    throw new Error(`[DOCS_CODE_CONFIG] ${label} must be an object`);
  return value as Fields;
}
function defined(value: Fields): Fields {
  return Object.fromEntries(
    Object.entries(value).filter(([, item]) => item !== undefined)
  );
}
function tokens(legacy: unknown, current: unknown): Fields | undefined {
  if (legacy === undefined && current === undefined) return undefined;
  const before =
    legacy === undefined ? {} : object(legacy, 'snippets.highlight.tokens');
  const after = current === undefined ? {} : object(current, 'code.tokens');
  const result = { ...defined(before), ...defined(after) };
  for (const mode of ['light', 'dark']) {
    if (before[mode] === undefined && after[mode] === undefined) continue;
    result[mode] = {
      ...(before[mode] === undefined
        ? {}
        : defined(object(before[mode], `snippets.highlight.tokens.${mode}`))),
      ...(after[mode] === undefined
        ? {}
        : defined(object(after[mode], `code.tokens.${mode}`)))
    };
  }
  return result;
}

/** Resolve once before native prepare; every code path consumes this same palette. */
export function resolveDocsCodeTheme(
  site: SiteConfig,
  snippets?: DocsSnippetsOptions
) {
  const legacy =
    snippets?.highlight === undefined
      ? {}
      : object(snippets.highlight, 'snippets.highlight');
  const current = site.code === undefined ? {} : object(site.code, 'code');
  const code: Fields = {};
  for (const key of ['light', 'dark'])
    if (legacy[key] !== undefined) code[key] = legacy[key];
  // Preserve every own JSON field for the native validator, including __proto__.
  Object.defineProperties(
    code,
    Object.getOwnPropertyDescriptors(defined(current))
  );
  const overrides = tokens(legacy.tokens, current.tokens);
  if (overrides !== undefined) code.tokens = overrides;
  // Preserve unknown palette fields for the authoritative Rust parser to reject.
  const canonical = code as CodeThemeConfig;
  return {
    site: { ...site, code: canonical },
    highlight: {
      ...legacy,
      ...canonical,
      representation: site.build?.highlightRepresentation ?? 'jsx'
    }
  };
}
