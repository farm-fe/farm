import { SecureServerOptions } from 'node:http2';
import { Server } from '../index.js';

import type { OutgoingHttpHeaders } from 'http';
import type cors from '@koa/cors';
import { WatchOptions } from 'chokidar';
import { Middleware } from 'koa';
import type { Config } from '../types/binding.js';
import type { RustPlugin } from '../plugin/rust/index.js';
import type { JsPlugin } from '../plugin/type.js';
import type { Options } from 'http-proxy-middleware';
import type { Logger } from '../utils/index.js';
import { HmrOptions } from '../newServer/index.js';

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
  https?: SecureServerOptions;
  protocol?: 'http' | 'https';
  hostname?: { name: string; host: string | undefined };
  // http2?: boolean;
  hmr?: boolean | UserHmrConfig;
  proxy?: Record<string, Options>;
  strictPort?: boolean;
  open?: boolean;
  host?: string | boolean;
  cors?: boolean | cors.Options;
  // whether to serve static assets in spa mode, default to true
  spa?: boolean;
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
    hmr?: UserHmrConfig;
  }
>;

export interface NormalizedConfig {
  compilationConfig: Config;
  serverConfig?: NormalizedServerConfig;
}

export interface UserHmrConfig extends HmrOptions {
  watchOptions?: WatchOptions;
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
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface ResolvedCompilation extends Exclude<Config['config'], undefined> {
  external?: string[];
}

export interface ResolvedUserConfig extends UserConfig {
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
  logger?: Logger;
  config?: string;
  configFile?: string;
  configPath?: string;
  compilation?: Config['config'];
  mode?: string;
  root?: string;
  server?: FarmCLIServerOptions;
  clearScreen?: boolean;
}

export type DevServerMiddleware = (context: Server) => Middleware | undefined;

export interface Alias {
  // TODO support RegExp
  find: string;
  replacement: string;
}
