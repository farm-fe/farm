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
  runtime_isolate: 'runtime.isolate'
};

export const FARM_RUST_PLUGIN_FUNCTION_ENTRY = 'func.js';

const { version } = JSON.parse(
  readFileSync(new URL('../../package.json', import.meta.url)).toString()
);

export const VERSION = version;
