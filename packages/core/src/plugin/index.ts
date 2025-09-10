export * from './js/index.js';
export * from './rust/index.js';

import { isAbsolute, relative } from 'path';
import {
  CompilationMode,
  ConfigEnv,
  ResolvedUserConfig,
  type UserConfig
} from '../config/index.js';
import { isArray, isObject } from '../utils/index.js';
import merge from '../utils/merge.js';
import { JS_PLUGIN_BRIDGE_HOOKS } from './constant.js';
import { convertPlugin, handleVitePlugins } from './js/index.js';
import { rustPluginResolver } from './rust/index.js';
import type { JsPlugin } from './type.js';

export async function resolveVitePlugins(
  config: UserConfig,
  mode: CompilationMode
) {
  const plugins = config?.vitePlugins?.filter(Boolean) ?? [];
  if (!plugins.length) return [];

  return handleVitePlugins(plugins, config, mode);
}

export async function resolveFarmPlugins(config: UserConfig) {
  const plugins = config.plugins ?? [];

  if (!plugins.length) {
    return {
      rustPlugins: [],
      jsPlugins: []
    };
  }

  const rustPlugins = [];

  const jsPlugins: JsPlugin[] = [];

  for (let plugin of plugins) {
    if (!plugin) {
      continue;
    }

    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(
        await rustPluginResolver(plugin as string, config.root ?? process.cwd())
      );
    } else if (isObject(plugin)) {
      // @ts-ignore
      plugin = convertPlugin(plugin as unknown as JsPlugin);
      jsPlugins.push(plugin as unknown as JsPlugin);
    } else if (isArray(plugin)) {
      for (let pluginNestItem of plugin as JsPlugin[]) {
        // @ts-ignore
        pluginNestItem = convertPlugin(pluginNestItem as JsPlugin);
        jsPlugins.push(pluginNestItem as JsPlugin);
      }
    } else {
      throw new Error(
        `plugin ${plugin} is not supported, Please pass the correct plugin type`
      );
    }
  }

  return {
    rustPlugins,
    jsPlugins
  };
}

// resolve promise plugins
export async function resolveAsyncPlugins<T>(arr: T[]): Promise<T[]> {
  return arr.reduce<Promise<T[]>>(async (acc, current) => {
    const flattenedAcc = await acc;

    if (current instanceof Promise) {
      const resolvedElement = await current;
      return flattenedAcc.concat(resolvedElement);
    } else if (Array.isArray(current)) {
      const flattenedArray = await resolveAsyncPlugins(current);
      return flattenedAcc.concat(flattenedArray);
    } else {
      return flattenedAcc.concat(current);
    }
  }, Promise.resolve([]));
}

export async function resolveConfigHook(
  config: UserConfig,
  configEnv: ConfigEnv,
  plugins: JsPlugin[]
): Promise<UserConfig> {
  let conf = config;

  const uniqueVitePlugins = new Map<string, JsPlugin>();

  for (const p of plugins) {
    const pluginName = p.name;

    if (!uniqueVitePlugins.has(pluginName)) {
      uniqueVitePlugins.set(pluginName, p);
    }
  }

  for (const p of uniqueVitePlugins.values()) {
    if (p.config) {
      const res = await p.config(conf, configEnv);

      if (res) {
        conf = merge(conf, res);
      }
    }
  }

  return conf;
}

export async function resolveConfigResolvedHook(
  config: ResolvedUserConfig,
  plugins: JsPlugin[]
) {
  for (const p of plugins) {
    if (p.configResolved) {
      await p.configResolved(config);
    }
  }

  // normalize js plugins filters to transform resolved paths from absolute to relative
  for (const p of plugins) {
    for (const key of Object.keys(p)) {
      const hook: any = p[key as keyof JsPlugin];

      if (
        typeof hook !== 'object' ||
        !['load', 'transform', 'processModule', 'freezeModule'].includes(key)
      ) {
        continue;
      }

      if (hook?.filters?.resolvedPaths?.length && config.root) {
        // Convert absolute paths to relative paths
        hook.filters.resolvedPaths = hook.filters.resolvedPaths.map(
          (p: string) =>
            isAbsolute(p) && !p.startsWith('\\') ? relative(config.root, p) : p
        );
      }
    }
  }
}

const processedPlugin: WeakSet<any> = new WeakSet<any>();

function proxyContext<V extends JsPlugin>(plugins: V[]): V[] {
  for (const plugin of plugins) {
    for (const key of Object.keys(plugin)) {
      const hook = plugin[key as keyof JsPlugin];
      if (JS_PLUGIN_BRIDGE_HOOKS.has(key)) {
        const [callback, set] =
          typeof hook === 'object'
            ? ([
                hook.executor,
                (callback: any) => (hook.executor = callback)
              ] as const)
            : // @ts-ignore
              ([hook, (callback: any) => (plugin[key] = callback)] as const);

        if (typeof callback === 'function') {
          function proxyHookCallback(...args: any) {
            let context = args[1];

            if (context) {
              const proxyKeys = [
                'readCache',
                'writeCache',
                'readCacheByScope'
              ] as const;
              const nativeFunction = proxyKeys.reduce(
                (res, k) => ({
                  [k]: context[k],
                  ...res
                }),
                {} as Record<(typeof proxyKeys)[number], Function>
              );
              function tryProxyMethod(
                name: keyof typeof nativeFunction,
                fn: any
              ) {
                if (
                  nativeFunction[name] != null &&
                  processedPlugin.has(nativeFunction[name])
                ) {
                  return;
                }

                context[name] = fn;
              }

              tryProxyMethod('readCache', function (...args: any[]) {
                const result = nativeFunction['readCache'].apply(this, args);

                const returnValue = (value: any) => {
                  return value && typeof value === 'string'
                    ? JSON.parse(value)
                    : value;
                };

                if (result && typeof result.then === 'function') {
                  return result.then(returnValue);
                }

                if (result && typeof result === 'string') {
                  return returnValue(result);
                }

                return result;
              });

              tryProxyMethod('writeCache', function (...args: any[]) {
                if (args[1] && typeof args[1] !== 'string') {
                  args[1] = JSON.stringify(args[1]);
                }
                return nativeFunction['writeCache'].apply(this, args);
              });

              tryProxyMethod('readCacheByScope', function (...args: any[]) {
                const result = nativeFunction['readCacheByScope'].apply(
                  this,
                  args
                );
                const returnValue = (result: any) => {
                  if (!Array.isArray(result)) {
                    return result;
                  }

                  return result.map((item) =>
                    item && typeof item === 'string' ? JSON.parse(item) : item
                  );
                };

                if (result && typeof result.then === 'function') {
                  return result.then(returnValue);
                }

                if (result) {
                  return returnValue(result);
                }

                return result;
              });
            }

            // @ts-ignore
            return callback.apply(this, args);
          }

          set(proxyHookCallback);
        }
      }
    }
  }

  return plugins;
}

export function getSortedPlugins(plugins: readonly JsPlugin[]): JsPlugin[] {
  // TODO The priority needs to be redefined.
  const DEFAULT_PRIORITY = 100;

  const sortedPlugins = plugins
    .filter(
      (plugin): plugin is JsPlugin & { priority: number } =>
        typeof plugin === 'object' && typeof plugin.priority === 'number'
    )
    .sort((a, b) => b.priority - a.priority);

  const prePlugins = sortedPlugins.filter(
    (plugin) => plugin?.priority > DEFAULT_PRIORITY
  );

  const postPlugins = sortedPlugins.filter(
    (plugin) => plugin?.priority < DEFAULT_PRIORITY
  );

  const normalPlugins = plugins.filter(
    (plugin) =>
      (typeof plugin === 'object' && typeof plugin.priority !== 'number') ||
      plugin?.priority === DEFAULT_PRIORITY
  );

  return proxyContext([...prePlugins, ...normalPlugins, ...postPlugins]);
}

export function getSortedPluginHooksBindThis(
  plugins: JsPlugin[],
  hookName: keyof JsPlugin
) {
  plugins = getSortedPlugins(plugins);
  return plugins
    .map((p: JsPlugin) =>
      typeof p[hookName] === 'function' ? p[hookName].bind(p) : p[hookName]
    )
    .filter(Boolean);
}
