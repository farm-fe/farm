import { isArray, isObject } from '../utils/index.js';
import { convertPlugin, handleVitePlugins } from './js/index.js';
import { rustPluginResolver } from './rust/index.js';

import type { JsPlugin } from './type.js';
import type { Config } from '../../binding/index.js';
import {
  ConfigEnv,
  mergeConfiguration,
  resolveAsyncPlugins,
  type UserConfig
} from '../config/index.js';

export * from './js/index.js';
export * from './rust/index.js';

/**
 * resolvePlugins split / jsPlugins / rustPlugins
 * @param config
 */
export async function resolveAllPlugins(
  resolvedConfig: Config['config'],
  userConfig: UserConfig
) {
  const plugins = userConfig.plugins ?? [];
  const vitePlugins = (userConfig.vitePlugins ?? []).filter(Boolean);

  if (!plugins.length && !vitePlugins?.length) {
    return {
      rustPlugins: [],
      rawJsPlugins: [],
      resolvedConfig
    };
  }

  const rustPlugins = [];

  const vitePluginAdapters: JsPlugin[] = handleVitePlugins(
    vitePlugins,
    userConfig,
    resolvedConfig
  );

  const jsPlugins: JsPlugin[] = [];

  for (const plugin of plugins) {
    if (
      typeof plugin === 'string' ||
      (isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(
        await rustPluginResolver(plugin as string, resolvedConfig?.root)
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
  // const config = await resolveConfigHook(resolvedConfig, jsPlugins);
  const filterPlugins = filterPluginByName(jsPlugins);

  const sortJsPlugins = getSortedPlugins(filterPlugins);

  const rawJsPlugins = (await resolveAsyncPlugins(
    sortJsPlugins || []
  )) as JsPlugin[];

  // call user config hooks
  // for (const jsPlugin of jsPlugins) {
  //   resolvedConfig = (await jsPlugin.config?.(resolvedConfig)) ??
  //     resolvedConfig;
  // }

  return {
    rustPlugins,
    rawJsPlugins,
    resolvedConfig
  };
}

function filterPluginByName(plugins: JsPlugin[]) {
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

  for (const p of plugins) {
    const hook = p.config;

    if (hook) {
      const res = await p.config(conf, configEnv);
      if (res) {
        conf = mergeConfiguration(conf, res);
      }
    }
  }
  return conf;
}

export async function resolveConfigResolvedHook(
  config: any,
  plugins: JsPlugin[]
): Promise<UserConfig> {
  const conf = config;

  for (const p of plugins) {
    const hook = p.configResolved;
    if (hook) {
      await p.configResolved(conf);
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
  const normalPlugins = plugins.filter(Boolean);

  return [...prePlugins, ...normalPlugins, ...postPlugins];
}
