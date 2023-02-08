import { createRequire } from 'module';
import path from 'path';

export type RustPlugin =
  | string
  | [
      string,
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      Record<string, any>
    ];

/**
 * Resolve the binary plugin file, return [filePath, jsonStringifiedOptions]
 * @param plugin rust plugin config
 */
export function rustPluginResolver(
  plugin: RustPlugin,
  root: string
): [string, string] {
  let pluginPath: string, options: string;

  if (typeof plugin === 'string') {
    pluginPath = plugin;
    options = '{}';
  }
  if (Array.isArray(plugin) && plugin.length === 2) {
    pluginPath = plugin[0];
    options = JSON.stringify(plugin[1]);
  } else {
    throw new Error(
      'Invalid config: [plugins]. A rust plugin must be a string, or [string, Record<string, any>]'
    );
  }

  // not relative path, treat it as a package
  if (!path.isAbsolute(pluginPath) && !pluginPath.startsWith('.')) {
    const require = createRequire(root);
    pluginPath = require.resolve(pluginPath);
  }

  return [pluginPath, options];
}
