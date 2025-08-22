import { readFileSync } from 'node:fs';
import {
  HmrOptions,
  NormalizedServerConfig,
  ResolvedCompilation
} from './types.js';

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
  assets_mode: 'assets.mode',
  output_ascii_only: 'output.ascii_only'
};

export const FARM_RUST_PLUGIN_FUNCTION_ENTRY = 'func.js';

const { version } = JSON.parse(
  readFileSync(new URL('../../package.json', import.meta.url)).toString()
);

export const VERSION = version;

export const ENV_PRODUCTION = 'production';
export const ENV_DEVELOPMENT = 'development';

export const DEFAULT_HMR_OPTIONS: Required<HmrOptions> = {
  host: 'localhost',
  port:
    (process.env.FARM_DEFAULT_HMR_PORT &&
      Number(process.env.FARM_DEFAULT_HMR_PORT)) ??
    undefined,
  path: '/__hmr',
  overlay: true,
  clientPort: 9000,
  timeout: 0,
  server: null,
  protocol: ''
};

export const DEFAULT_DEV_SERVER_OPTIONS: NormalizedServerConfig = {
  headers: {},
  port:
    (process.env.FARM_DEFAULT_SERVER_PORT &&
      Number(process.env.FARM_DEFAULT_SERVER_PORT)) ||
    9000,
  https: undefined,
  protocol: 'http',
  hostname: {
    name: 'localhost',
    host: undefined
  },
  allowedHosts: [],
  host: 'localhost',
  proxy: undefined,
  hmr: DEFAULT_HMR_OPTIONS,
  middlewareMode: false,
  open: false,
  strictPort: false,
  cors: false,
  middlewares: [],
  appType: 'spa',
  writeToDisk: false,
  origin: '',
  preview: {
    host: 'localhost',
    headers: {},
    port:
      (process.env.FARM_DEFAULT_SERVER_PORT &&
        Number(process.env.FARM_DEFAULT_SERVER_PORT)) ||
      1911,
    strictPort: false,
    https: undefined,
    distDir: 'dist',
    open: false,
    cors: false,
    proxy: undefined
  }
};

export const DEFAULT_COMPILATION_OPTIONS: Partial<ResolvedCompilation> = {
  output: {
    path: './dist'
  },
  sourcemap: true,
  resolve: {
    extensions: [
      'tsx',
      'mts',
      'cts',
      'ts',
      'jsx',
      'mjs',
      'js',
      'cjs',
      'json',
      'html',
      'css',
      'mts',
      'cts'
    ]
  }
};
