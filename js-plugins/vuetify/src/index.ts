import type { JsPlugin } from '@farmfe/core';
import {
  generateImports,
  includes,
  isObject,
  normalizePath,
  resolveVuetifyBase,
  type Options as VuetifyOptions
} from '@vuetify/loader-shared';
import path from 'upath';

interface Options extends VuetifyOptions {}

type VueFilterPattern = string | RegExp | (string | RegExp)[];

interface VuePluginOptions {
  include?: VueFilterPattern;
  exclude?: VueFilterPattern;
}

type ViteVuePlugin = JsPlugin & {
  api?: {
    options?: VuePluginOptions;
  };
};

const SUPPORTED_VITE_VUE_PLUGIN_NAMES = new Set(['unplugin-vue', 'vite:vue']);
const FARM_PLUGIN_VUE_PACKAGE = '@farmfe/plugin-vue';
const DEFAULT_VUE_INCLUDE = /\.vue$/;

function isSubdir(root: string, test: string) {
  const relative = path.relative(root, test);
  return relative && !relative.startsWith('..') && !path.isAbsolute(relative);
}

function filterToRegExp(filter: unknown, fallback: RegExp | null) {
  if (!filter) return fallback;
  if (filter instanceof RegExp) return filter;
  if (typeof filter === 'string') return new RegExp(filter);
  if (Array.isArray(filter)) {
    const sources = filter
      .map((item) => {
        if (item instanceof RegExp) return item.source;
        if (typeof item === 'string') return item;
        return null;
      })
      .filter(Boolean);

    return sources.length ? new RegExp(sources.join('|')) : fallback;
  }
  return fallback;
}

function isFarmPluginVuePath(pluginPath: string) {
  const normalizedPath = normalizePath(pluginPath);
  return (
    normalizedPath === FARM_PLUGIN_VUE_PACKAGE ||
    normalizedPath.includes('/@farmfe/plugin-vue') ||
    normalizedPath.includes('/rust-plugins/vue/') ||
    normalizedPath.includes('/farm-plugin-vue')
  );
}

function parseRustPluginOptions(
  options: unknown
): VuePluginOptions | undefined {
  if (!options) return undefined;
  if (typeof options === 'string') {
    try {
      return JSON.parse(options);
    } catch {
      return undefined;
    }
  }
  if (isObject(options)) return options as VuePluginOptions;
  return undefined;
}

function getFarmPluginVueOptions(config: any): VuePluginOptions | undefined {
  for (const plugin of config.plugins ?? []) {
    if (typeof plugin === 'string' && isFarmPluginVuePath(plugin)) return {};
    if (Array.isArray(plugin) && isFarmPluginVuePath(plugin[0])) {
      return parseRustPluginOptions(plugin[1]) ?? {};
    }
  }

  for (const [pluginPath, options] of config.rustPlugins ?? []) {
    if (isFarmPluginVuePath(pluginPath)) {
      return parseRustPluginOptions(options) ?? {};
    }
  }

  return undefined;
}

const PLUGIN_VIRTUAL_PREFIX = 'virtual:';
const PLUGIN_VIRTUAL_NAME = 'plugin-vuetify';
const VIRTUAL_MODULE_ID = `${PLUGIN_VIRTUAL_PREFIX}${PLUGIN_VIRTUAL_NAME}`;

export default function farmPlugin(_options?: Options): JsPlugin[] {
  let include: RegExp | null;
  let exclude: RegExp | null;

  const options = _options || {};
  options.autoImport ??= true;
  options.styles ??= true;

  const vuetifyBase = resolveVuetifyBase();
  let configFile: string | undefined;
  const tempFiles = new Map();

  const importPlugin: JsPlugin = {
    name: 'js-plugin:vuetify:import',
    priority: 40,

    configResolved(config) {
      const vitePlugins = (config.vitePlugins ?? []) as ViteVuePlugin[];

      const vuePlugin = vitePlugins.find((plugin: ViteVuePlugin) =>
        SUPPORTED_VITE_VUE_PLUGIN_NAMES.has(plugin.name)
      );
      const vueOptions =
        vuePlugin?.api?.options ?? getFarmPluginVueOptions(config);

      if (!vueOptions) {
        throw new Error(
          'No Vue plugin found, please install @farmfe/plugin-vue, @vitejs/plugin-vue or unplugin-vue.'
        );
      }

      if (vuePlugin && vuePlugin.api?.options === undefined) {
        throw new Error('Vue plugin options not found.');
      }

      include = filterToRegExp(vueOptions.include, DEFAULT_VUE_INCLUDE);
      exclude = filterToRegExp(vueOptions.exclude, null);
    },
    transform: {
      filters: {
        resolvedPaths: ['\\.vue$']
      },
      async executor(param) {
        const { content, query, resolvedPath } = param;
        const isVueVirtual =
          query && query.find(([key, _value]) => key === 'vue') !== undefined;
        const isVueFile =
          !isVueVirtual &&
          include?.test(resolvedPath) &&
          (!exclude || !exclude.test(resolvedPath)) &&
          !/^import { render as _sfc_render } from ".*"$/m.test(content);
        const isVueTemplate =
          isVueVirtual &&
          query.find(([key, value]) => {
            const matchType =
              key === 'type' && (value === 'template' || value === 'script');
            const matchSetup = key === 'setup' && value === 'true';
            return matchType || matchSetup;
          });
        if (isVueFile || isVueTemplate) {
          const { code: imports, source } = generateImports(content, options);
          return {
            content: source + imports,
            sourceMap: null
          };
        }
        return null;
      }
    }
  };

  const stylesPlugin: JsPlugin = {
    name: 'js-plugin:vuetify:styles',
    priority: 120,

    configResolved(config) {
      if (isObject(options.styles)) {
        if (path.isAbsolute(options.styles.configFile)) {
          configFile = options.styles.configFile;
        } else {
          configFile = path.join(
            config.root || process.cwd(),
            options.styles.configFile
          );
        }
      }
    },
    resolve: {
      filters: {
        importers: ['.*'],
        sources: ['\\.css$', 'vuetify/styles']
      },
      async executor(param) {
        const { source, importer } = param;

        if (
          source === 'vuetify/styles' ||
          (importer &&
            source.endsWith('.css') &&
            isSubdir(vuetifyBase, path.isAbsolute(source) ? source : importer))
        ) {
          if (options.styles === 'none') {
            return { resolvedPath: `${PLUGIN_VIRTUAL_PREFIX}__void__` };
          }
          if (options.styles === 'sass') {
            const target = source.replace(/\.css$/, '.sass');
            return {
              resolvedPath: target
            };
          }
          if (isObject(options.styles)) {
            // Vite port plugin resolves the file manually by calling `this.resolve`,
            // this is a workaround and may not work properly.
            const target = source.replace(/\.css$/, '.sass');
            const file = path.relative(path.join(vuetifyBase, 'lib'), target);
            const contents = `@use "${normalizePath(configFile ?? '')};\n@use "${normalizePath(target)}";`;
            tempFiles.set(file, contents);
            return { resolvedPath: `${VIRTUAL_MODULE_ID}:${file}` };
          }
        } else if (source.startsWith(`/${PLUGIN_VIRTUAL_NAME}:`)) {
          return { resolvedPath: PLUGIN_VIRTUAL_PREFIX + source.slice(1) };
        } else if (source.startsWith(`/@id/__x00__${PLUGIN_VIRTUAL_NAME}:`)) {
          return { resolvedPath: PLUGIN_VIRTUAL_PREFIX + source.slice(12) };
        } else if (source.startsWith(`/${VIRTUAL_MODULE_ID}:`)) {
          return { resolvedPath: source.slice(1) };
        }

        return null;
      }
    },
    load: {
      filters: {
        resolvedPaths: [`^${PLUGIN_VIRTUAL_PREFIX}`, `^${VIRTUAL_MODULE_ID}`]
      },
      async executor(param) {
        const id = param.moduleId;
        if (new RegExp(`^${PLUGIN_VIRTUAL_PREFIX}__void__(\\?.*)?$`).test(id)) {
          return {
            content: '',
            moduleType: 'css'
          };
        }
        if (id.startsWith(`${VIRTUAL_MODULE_ID}`)) {
          const match = new RegExp(`^${VIRTUAL_MODULE_ID}:(.*?)(\\?.*)?$`).exec(
            id
          );
          const content = match ? tempFiles.get(match[1]) : null;
          return content ? { content, moduleType: 'css' } : null;
        }
        return null;
      }
    }
  };

  const plugins = [];
  if (options.autoImport) {
    plugins.push(importPlugin);
  }
  if (includes(['none', 'sass'], options.styles) || isObject(options.styles)) {
    plugins.push(stylesPlugin);
  }

  return plugins;
}

export { transformAssetUrls } from '@vuetify/loader-shared';
