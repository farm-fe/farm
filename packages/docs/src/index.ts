import type { UserConfig } from '@farmfe/core';
import docs from '@farmfe/plugin-docs';
import type { DocsSnippetsOptions } from '@farmfe/plugin-docs-snippets';
import type { SiteConfig } from './config.generated.js';
import { resolveDocsCodeTheme } from './highlight-config.js';

export type { DocsSnippetsOptions } from '@farmfe/plugin-docs-snippets';

export type * from './config.generated.js';
export type { DocsDevelopmentHandle } from './development.js';
export { startDocsDevelopment } from './development.js';
export type NativePlugin = string | [string, unknown];
export type DocsFarmConfig = Omit<
  UserConfig,
  'workflow' | 'plugins' | 'vitePlugins' | 'jsPlugins'
> & {
  plugins?: NativePlugin[];
  /** Static snippet compilation options, validated by the native plugin. */
  snippets?: DocsSnippetsOptions;
};

/** Composition only: configuration and compilation semantics are implemented in Rust. */
export function defineDocsConfig(
  site: SiteConfig,
  farm: DocsFarmConfig = {}
): UserConfig {
  assertNativePlugins(farm);
  const { site: resolvedSite } = resolveDocsCodeTheme(site, farm.snippets);
  const { snippets: _snippets, ...coreFarm } = farm;
  return {
    ...coreFarm,
    compilation: { ...farm.compilation, input: { docs: 'docs:site' } },
    plugins: [docs({ site: resolvedSite }), ...(farm.plugins ?? [])],
    workflow: {
      module: '@farmfe/docs/workflow',
      options: { site: resolvedSite, farm }
    }
  };
}

export function assertNativePlugins(farm: DocsFarmConfig): void {
  const input = farm as UserConfig & { jsPlugins?: unknown[] };
  if (
    input.workflow ||
    input.vitePlugins?.some(Boolean) ||
    input.jsPlugins?.length ||
    farm.plugins?.some(
      (plugin) =>
        typeof plugin !== 'string' &&
        !(
          Array.isArray(plugin) &&
          plugin.length === 2 &&
          typeof plugin[0] === 'string'
        )
    )
  ) {
    throw new Error(
      '[DOCS_RUST_ONLY] Farm Docs accepts native Rust plugins only'
    );
  }
}
