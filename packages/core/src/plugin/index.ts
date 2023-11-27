import { isArray, isObject } from '../utils/index.js';
import { convertPlugin, handleVitePlugins } from './js/index.js';
import { rustPluginResolver } from './rust/index.js';

import type { JsPlugin } from './type.js';
import type { Config } from '../../binding/index.js';
import { ConfigEnv, type UserConfig } from '../config/index.js';

export * from './js/index.js';
export * from './rust/index.js';

/**
 * resolvePlugins split / jsPlugins / rustPlugins
 * @param config
 */
export async function resolveAllPlugins(
  finalConfig: Config['config'],
  userConfig: UserConfig
) {
  const plugins = userConfig.plugins ?? [];
  const vitePlugins = (userConfig.vitePlugins ?? []).filter(Boolean);

  if (!plugins.length && !vitePlugins?.length) {
    return {
      rustPlugins: [],
      jsPlugins: [],
      finalConfig
    };
  }

  const rustPlugins = [];

  const vitePluginAdapters: JsPlugin[] = handleVitePlugins(
    vitePlugins,
    userConfig,
    finalConfig
  );

  const jsPlugins: JsPlugin[] = [];

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(
        await rustPluginResolver(plugin as string, finalConfig.root)
      );
    } else if (isObject(plugin)) {
      convertPlugin(plugin as unknown as JsPlugin);
      jsPlugins.push(plugin as unknown as JsPlugin);
    } else if (isArray(plugin)) {
      for (const pluginNestItem of plugin as JsPlugin[]) {
        convertPlugin(pluginNestItem as JsPlugin);
        jsPlugins.push(pluginNestItem as JsPlugin);
      }
    } else {
      throw new Error(
        `plugin ${plugin} is not supported, Please pass the correct plugin type`
      );
    }
  }
  // vite plugins execute after farm plugins by default.
  jsPlugins.push(...vitePluginAdapters);

  // call user config hooks
  for (const jsPlugin of jsPlugins) {
    finalConfig = (await jsPlugin.config?.(finalConfig)) ?? finalConfig;
  }

  return {
    rustPlugins,
    jsPlugins,
    finalConfig
  };
}

export async function resolveJsPlugins(
  finalConfig: Config['config'],
  userConfig: UserConfig
) {
  const plugins = userConfig.plugins ?? [];
  const vitePlugins = (userConfig.vitePlugins ?? []).filter(Boolean);

  if (!plugins.length && !vitePlugins?.length) {
    return {
      jsPlugins: [],
      finalConfig
    };
  }

  const vitePluginAdapters: JsPlugin[] = handleVitePlugins(
    vitePlugins,
    userConfig,
    finalConfig
  );

  const jsPlugins: JsPlugin[] = [];

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      // Ignore or handle the string or specific array format
      continue;
    }
    if (isObject(plugin)) {
      convertPlugin(plugin as unknown as JsPlugin);
      jsPlugins.push(plugin as unknown as JsPlugin);
    } else if (isArray(plugin)) {
      for (const pluginNestItem of plugin as JsPlugin[]) {
        convertPlugin(pluginNestItem as JsPlugin);
        jsPlugins.push(pluginNestItem as JsPlugin);
      }
    } else {
      throw new Error(
        `plugin ${plugin} is not supported, Please pass the correct plugin type`
      );
    }
  }
  // vite plugins execute after farm plugins by default.
  jsPlugins.push(...vitePluginAdapters);

  return {
    jsPlugins,
    finalConfig
  };
}

export async function resolveRustPlugins(
  compilationConfig: Config['config'],
  userConfig: UserConfig
) {
  const plugins = userConfig.plugins ?? [];

  if (!plugins.length) {
    return {
      rustPlugins: []
    };
  }

  const rustPlugins = [];

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(
        await rustPluginResolver(plugin as string, compilationConfig.root)
      );
    }
  }

  return {
    rustPlugins
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

export function filterPluginByName(plugins: JsPlugin[]) {
  const uniqueNamesSet = new Set();

  // 使用 Array.filter 进行过滤
  const filteredArray = plugins.filter((obj) => {
    // 如果 Set 中没有出现过当前对象的 name 值，加入 Set，并保留该对象
    if (!uniqueNamesSet.has(obj.name)) {
      uniqueNamesSet.add(obj.name);
      return true;
    }
    // 如果 Set 中已经出现过相同的 name 值，过滤掉该对象
    return false;
  });

  return filteredArray;
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
    const hook = p.config;

    if (hook) {
      const res = await p.config(conf, configEnv);

      if (res) {
        conf = mergeConfig(conf, res);
      }
    }
  }

  return conf;
}

export function mergeConfig(
  a: Record<string, any>,
  b: Record<string, any>
): Record<string, any> {
  const result: Record<string, any> = { ...a };

  for (const key in b) {
    if (Object.prototype.hasOwnProperty.call(b, key)) {
      const value = b[key];

      if (value == null) {
        continue;
      }

      if (isArray(value)) {
        // 直接替换数组
        result[key] = value;
      } else if (isObject(value)) {
        // 递归替换对象
        result[key] = mergeConfig({}, value);
      } else {
        // 替换其他值
        result[key] = value;
      }
    }
  }

  return result;
}

export async function resolveConfigResolvedHook(
  config: any,
  plugins: JsPlugin[]
): Promise<UserConfig> {
  const conf = config;

  for (const p of plugins) {
    const hook = p.configResolved;
    if (hook) {
      await p.configResolved(conf.config);
    }
  }
  return conf;
}

export function getSortedPlugins(plugins: readonly JsPlugin[]): JsPlugin[] {
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
      typeof plugin === 'object' && typeof plugin.priority !== 'number'
  );

  return [...prePlugins, ...normalPlugins, ...postPlugins];
}
