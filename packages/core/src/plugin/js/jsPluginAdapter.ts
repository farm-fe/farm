import { toArray } from '../../utils/index.js';

export function adaptorVitePlugin<UserOptions = Record<string, never>>(
  factory: any
) {
  return (userOptions?: UserOptions) => {
    const meta: any = {
      framework: 'vite'
    };
    const rawPlugins = toArray(factory(userOptions!, meta));

    const plugins = rawPlugins.map((rawPlugin: any) => {
      const plugin = transformFarmPlugin(rawPlugin);

      return plugin;
    });

    return plugins.length === 1 ? plugins[0] : plugins;
  };
}

function transformFarmPlugin(plugin: any) {
  console.log(plugin);

  return {};
}
