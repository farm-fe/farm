import { convertPlugin } from './js/index.js';
import { rustPluginResolver } from './rust/index.js';
import type { JsPlugin } from './type.js';
import { ResolvedUserConfig, type UserConfig } from '../config/index.js';
import merge from '../utils/merge.js';

export * from './js/index.js';
export * from './rust/index.js';

// combing resolveFarmPlugin and resolveAsyncPlugin
export async function resolveFarmPluginsAndAsyncPlugins(config: UserConfig) {
  const plugins = config.plugins ?? [];

  if (!plugins.length) {
    return {
      rustPlugins: [],
      jsPlugins: []
    };
  }

  const rustPlugins = [];
  const jsPlugins: JsPlugin[] = [];

  for (const plugin of plugins) {
    if (!plugin) {
      continue;
    }

    if (
      typeof plugin === 'string' ||
      (Array.isArray(plugin) && typeof plugin[0] === 'string')
    ) {
      rustPlugins.push(
        await rustPluginResolver(plugin as string, config.root ?? process.cwd())
      );
    } else if (typeof plugin === 'object') {
      convertPlugin(plugin as unknown as JsPlugin);
      jsPlugins.push(plugin as unknown as JsPlugin);
    } else if (Array.isArray(plugin)) {
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

  const resolvedRustPlugins = await resolveAsyncPlugins(rustPlugins);
  const resolvedJsPlugins = await resolveAsyncPlugins(jsPlugins);

  return {
    rustPlugins: resolvedRustPlugins,
    jsPlugins: resolvedJsPlugins
  };
}

async function resolveAsyncPlugins<T>(arr: T[]): Promise<T[]> {
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
      const res = await p.config(conf);

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

  return [...prePlugins, ...normalPlugins, ...postPlugins];
}

export function getSortedPluginHooks(
  plugins: JsPlugin[],
  hookName: keyof JsPlugin
): any {
  return plugins.map((p: JsPlugin) => p[hookName]).filter(Boolean);
}
