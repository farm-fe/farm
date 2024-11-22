import { SecureServerOptions } from 'node:http2';

import { Server } from '../index.js';

import type { OutgoingHttpHeaders } from 'http';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import type { WatchOptions } from 'chokidar';
import type { RustPlugin } from '../plugin/rust/index.js';
import type { JsPlugin } from '../plugin/type.js';
import type { Config, CssConfig } from '../types/binding.js';
import type { Logger } from '../utils/index.js';

export interface HmrOptions {
  protocol?: string;
  host?: string;
  port?: number;
  clientPort?: number;
  path?: string;
  timeout?: number;
  overlay?: boolean;
  server?: Server;
}

export interface ConfigEnv {
  mode: string;
  command: string;
  isPreview: boolean;
}

export type UserConfigFnPromise = (env: ConfigEnv) => Promise<UserConfig>;
export type UserConfigFn = (env: ConfigEnv) => UserConfig | Promise<UserConfig>;
export type UserConfigFnObject = (env: ConfigEnv) => UserConfig;

export type UserConfigExport =
  | UserConfig
  | Promise<UserConfig>
  | UserConfigFnObject
  | UserConfigFnPromise
  | UserConfigFn;

export interface UserServerConfig {
  headers?: OutgoingHttpHeaders | undefined;
  port?: number;
  https?: HttpsServerOptions;
  protocol?: 'http' | 'https';
  hostname?: { name: string; host: string | undefined };
  // http2?: boolean;
  hmr?: boolean | HmrOptions;
  proxy?: Record<string, any>;
  strictPort?: boolean;
  open?: boolean;
  host?: string | boolean;
  cors?: boolean | any;
  // whether to serve static assets in spa mode, default to true
  appType?: 'spa' | 'mpa' | 'custom';
  middlewares?: DevServerMiddleware[];
  middlewareMode?: boolean | string;
  writeToDisk?: boolean;
}

export interface UserPreviewServerConfig {
  // write static output file
  output?: { path?: string; publicPath?: string };
  distDir?: string;
  https?: SecureServerOptions;
  port?: number;
  host?: string | boolean;
}

export type NormalizedServerConfig = Required<
  Omit<UserServerConfig, 'hmr'> & {
    hmr?: HmrOptions;
  }
>;

export interface NormalizedConfig {
  compilationConfig: Config;
  serverConfig?: NormalizedServerConfig;
}

type InternalConfig = Config['config'] extends undefined
  ? object
  : Required<Config>['config'];

type AvailableUserConfigKeys = Exclude<
  keyof InternalConfig,
  'configFilePath' | 'env' | 'coreLibPath' | 'root'
>;

export interface UserConfig {
  /** current root of this project, default to current working directory */
  root?: string;
  clearScreen?: boolean;
  envDir?: string;
  watch?: boolean | WatchOptions;
  envPrefix?: string | string[];
  publicDir?: string;
  /** js plugin(which is a javascript object) and rust plugin(which is string refer to a .farm file or a package) */
  plugins?: (RustPlugin | JsPlugin | JsPlugin[] | undefined | null | false)[];
  /** vite plugins */
  vitePlugins?: (
    | null
    | undefined
    | object
    | (() => { vitePlugin: any; filters: string[] })
  )[];
  /** config related to compilation */
  compilation?: Pick<InternalConfig, AvailableUserConfigKeys>;
  /** config related to dev server */
  server?: UserServerConfig;
  /** Files under this dir will always be treated as static assets. serve it in dev, and copy it to output.path when build */
  customLogger?: Logger;
}

interface ResolvedCss extends CssConfig {
  modules?: CssConfig['modules'] & {
    localsConversion?: never;
  };
}

interface ResolvedCss extends CssConfig {
  modules?: CssConfig['modules'] & {
    localsConversion?: never;
  };
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface ResolvedCompilation
  extends Exclude<Config['config'], undefined> {
  external?: string[];
  resolve?: {
    dedupe?: never;
  } & Config['config']['resolve'];
  assets?: Omit<Config['config']['assets'], 'mode'>;
  css?: ResolvedCss;
}

export interface ResolvedUserConfig extends UserConfig {
  root?: string;
  mode?: string;
  env?: Record<string, any>;
  envDir?: string;
  envFiles?: string[];
  envPrefix?: string | string[];
  configFilePath?: string;
  envMode?: string;
  configFileDependencies?: string[];
  compilation?: ResolvedCompilation;
  server?: NormalizedServerConfig;
  jsPlugins?: JsPlugin[];
  rustPlugins?: [string, string][];
  inlineConfig?: FarmCliOptions;
  logger?: Logger;
  customLogger?: Logger;
  watch?: boolean | WatchOptions;
}

export interface GlobalCliOptions {
  '--'?: string[];
  c?: boolean | string;
  config?: string;
  m?: string;
  mode?: 'development' | 'production';
}

export interface FarmCLIServerOptions {
  port?: number;
  open?: boolean;
  https?: SecureServerOptions;
  hmr?: boolean;
  host?: boolean | string;
  strictPort?: boolean;
}

export interface FarmCLIBuildOptions {
  outDir?: string;
  sourcemap?: boolean;
  minify?: boolean;
}

export interface FarmCLIPreviewOptions {
  open?: boolean;
  https?: SecureServerOptions;
  port?: number;
  host?: string | boolean;
}

export interface FarmCliOptions
  extends FarmCLIBuildOptions,
    FarmCLIPreviewOptions {
  config?: string;
  configFile?: string;
  compilation?: Config['config'];
  mode?: string;
  root?: string;
  server?: FarmCLIServerOptions;
  clearScreen?: boolean;
}

export type DevServerMiddleware = (context: Server) => any | undefined;

export interface Alias {
  find: string;
  replacement: string;
}
