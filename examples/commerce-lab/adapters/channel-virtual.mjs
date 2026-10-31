
export const channelModuleId = 'commerce-channel-capabilities.js';

/** Native factories are supplied by the Rust config so JS references load without native packages.
 * @returns {import('@farmfe/core').JsPlugin | [string, Record<string, string>]} */
export function channelVirtual(group, nativeFactory) {
  const version = process.env.COMMERCE_CHANNEL_RULE_VERSION ?? 'channels-2026-10';
  if (!/^[a-zA-Z0-9_-]+$/.test(version)) throw new Error('Invalid COMMERCE_CHANNEL_RULE_VERSION');
  const content = `export default ${JSON.stringify({ version, channels: ['Store', 'Social', 'Marketplace'] })};`;
  if (group !== 'js') {
    if (typeof nativeFactory !== 'function') throw new TypeError('channelVirtual requires a native plugin factory for the ' + group + ' group');
    return nativeFactory({ [channelModuleId]: content });
  }
  // Farm hook transport only: identical pre-generated JS, without a conversion workload.
  const resolved = `farm-virtual:${channelModuleId}`;
  return {
    name: 'commerce-js-channel-virtual',
    resolve: {
      filters: { sources: ['^commerce-channel-capabilities\\.js$'], importers: ['.*'] },
      executor() { return { resolvedPath: resolved }; },
    },
    load: {
      filters: { resolvedPaths: ['^farm-virtual:commerce-channel-capabilities\\.js$'] },
      executor() { return { content, moduleType: 'js' }; },
    },
  };
}
