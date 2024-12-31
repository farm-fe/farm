import { readFileSync } from 'node:fs';

export const DEFAULT_CONFIG_NAMES = [
  'farm.config.ts',
  'farm.config.js',
  'farm.config.cjs',
  'farm.config.mjs',
  'farm.config.cts',
  'farm.config.mts'
];

export const FARM_DEFAULT_NAMESPACE = 'FARM_DEFAULT_NAMESPACE';

export const CUSTOM_KEYS = {
  external_record: 'external.record',
  runtime_isolate: 'runtime.isolate',
  resolve_dedupe: 'resolve.dedupe',
  css_locals_conversion: 'css.modules.locals_conversion',
  partial_bundling_groups_enforce: 'partial_bundling.groups.enforce',
  assets_mode: 'assets.mode'
};

export const FARM_RUST_PLUGIN_FUNCTION_ENTRY = 'func.js';

const { version } = JSON.parse(
  readFileSync(new URL('../../package.json', import.meta.url)).toString()
);

export const VERSION = version;

export const ENV_PRODUCTION = 'production';
export const ENV_DEVELOPMENT = 'development';
