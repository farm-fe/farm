import {
  generateImports,
  resolveVuetifyBase,
  isObject,
  normalizePath,
  includes,
  type Options as VuetifyOptions,
} from "@vuetify/loader-shared";

import path from "upath";

import type { JsPlugin } from "@farmfe/core";

interface Options extends VuetifyOptions {}

function isSubdir(root: string, test: string) {
  const relative = path.relative(root, test);
  return relative && !relative.startsWith("..") && !path.isAbsolute(relative);
}

const PLUGIN_VIRTUAL_PREFIX = "virtual:";
const PLUGIN_VIRTUAL_NAME = "plugin-vuetify";
const VIRTUAL_MODULE_ID = `${PLUGIN_VIRTUAL_PREFIX}${PLUGIN_VIRTUAL_NAME}`;

export default function farmPlugin(_options?: Options): JsPlugin[] {
  let include: RegExp;
  let exclude: RegExp | null;

  const options = _options || {};
  options.autoImport ??= true;
  options.styles ??= true;

  const vuetifyBase = resolveVuetifyBase();
  let configFile: string | undefined;
  const tempFiles = new Map();

  const importPlugin: JsPlugin = {
    name: "js-plugin:vuetify:import",
    priority: 40,

    configResolved(config) {
      const { vitePlugins } = config;

      const vuePlugin = vitePlugins.find(
        (plugin: JsPlugin) =>
          plugin.name === "unplugin-vue" || plugin.name === "vite:vue",
      );

      if (!vuePlugin) {
        throw new Error(
          "No Vue plugin found, please install either @vitejs/plugin-vue nor unplugin-vue.",
        );
      }

      const vueOptions = vuePlugin.api.options;
      if (vueOptions === undefined) {
        throw new Error("Vue plugin options not found.");
      }

      include = vueOptions.include || /\.vue$/;
      exclude = vueOptions.exclude || null;
    },
    transform: {
      filters: {
        resolvedPaths: ["\\.vue$"],
      },
      async executor(param) {
        const { content, query, resolvedPath } = param;
        const isVueVirtual =
          query && query.find(([key, _value]) => key === "vue") !== undefined;
        const isVueFile =
          !isVueVirtual &&
          include.test(resolvedPath) &&
          (!exclude || !exclude.test(resolvedPath)) &&
          !/^import { render as _sfc_render } from ".*"$/m.test(content);
        const isVueTemplate =
          isVueVirtual &&
          query.find(([key, value]) => {
            const matchType =
              key === "type" && (value === "template" || value === "script");
            const matchSetup = key === "setup" && value === "true";
            return matchType || matchSetup;
          });
        if (isVueFile || isVueTemplate) {
          const { code: imports, source } = generateImports(content, options);
          return {
            content: source + imports,
            sourceMap: null,
          };
        }
        return null;
      },
    },
  };

  const stylesPlugin: JsPlugin = {
    name: "js-plugin:vuetify:styles",
    priority: 120,

    configResolved(config) {
      if (isObject(options.styles)) {
        if (path.isAbsolute(options.styles.configFile)) {
          configFile = options.styles.configFile;
        } else {
          configFile = path.join(
            config.root || process.cwd(),
            options.styles.configFile,
          );
        }
      }
    },
    resolve: {
      filters: {
        importers: [".*"],
        sources: ["\\.css$", "vuetify/styles"],
      },
      async executor(param) {
        const { source, importer } = param;

        if (
          source === "vuetify/styles" ||
          (importer &&
            source.endsWith(".css") &&
            isSubdir(vuetifyBase, path.isAbsolute(source) ? source : importer))
        ) {
          if (options.styles === "none") {
            return { resolvedPath: `${PLUGIN_VIRTUAL_PREFIX}__void__` };
          }
          if (options.styles === "sass") {
            const target = source.replace(/\.css$/, ".sass");
            return {
              resolvedPath: target,
            };
          }
          if (isObject(options.styles)) {
            // Vite port plugin resolves the file manually by calling `this.resolve`,
            // this is a workaround and may not work properly.
            const target = source.replace(/\.css$/, ".sass");
            const file = path.relative(path.join(vuetifyBase, "lib"), target);
            const contents = `@use "${normalizePath(configFile)};\n@use "${normalizePath(target)}";`;
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
      },
    },
    load: {
      filters: {
        resolvedPaths: [`^${PLUGIN_VIRTUAL_PREFIX}`, `^${VIRTUAL_MODULE_ID}`],
      },
      async executor(param) {
        const id = param.moduleId;
        if (new RegExp(`^${PLUGIN_VIRTUAL_PREFIX}__void__(\\?.*)?$`).test(id)) {
          return {
            content: "",
            moduleType: "css",
          };
        }
        if (id.startsWith(`${VIRTUAL_MODULE_ID}`)) {
          const content = tempFiles.get(
            new RegExp(`^${VIRTUAL_MODULE_ID}:(.*?)(\\?.*)?$`).exec(id)[1],
          );
          return content ? { content, moduleType: "css" } : null;
        }
        return null;
      },
    },
  };

  const plugins = [];
  if (options.autoImport) {
    plugins.push(importPlugin);
  }
  if (includes(["none", "sass"], options.styles) || isObject(options.styles)) {
    plugins.push(stylesPlugin);
  }

  return plugins;
}

export { transformAssetUrls } from "@vuetify/loader-shared";
