import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

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
export async function rustPluginResolver(
  plugin: RustPlugin,
  root: string
): Promise<[string, string]> {
  let pluginPath: string, options: string;

  if (typeof plugin === 'string') {
    pluginPath = plugin;
    options = '{}';
  } else if (Array.isArray(plugin) && plugin.length === 2) {
    pluginPath = plugin[0];
    options = JSON.stringify(plugin[1]);
  } else {
    throw new Error(
      'Invalid config: [plugins]. A rust plugin must be a string, or [string, Record<string, any>]'
    );
  }

  // not absolute path, treat it as a package
  if (!path.isAbsolute(pluginPath) && !pluginPath.startsWith('.')) {
    const require = createRequire(path.join(root, 'package.json'));
    pluginPath = require.resolve(pluginPath);
  }

  // a rust plugin' entry can be a .farm file or a .js file that exports the path to the binary
  if (!pluginPath.endsWith('.farm')) {
    // rust plugin should export a default string representing the path to the binary
    if (process.platform === 'win32') {
      pluginPath = (await import(pathToFileURL(pluginPath).toString())).default;
    } else {
      pluginPath = await import(pluginPath).then((m) => m.default);
    }

    // The entry js file should return { binary: string, options: Record<string, any> } when it's not string
    if (typeof pluginPath !== 'string') {
      const { binary, options: pluginOptions } = pluginPath as {
        binary: string;
        options: Record<string, any>;
      };
      options = JSON.stringify({
        ...pluginOptions,
        ...JSON.parse(options)
      });
      pluginPath = binary;
    }
  }

  return [pluginPath, options];
}
