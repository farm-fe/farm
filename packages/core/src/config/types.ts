import { Logger } from '../utils/logger.js';
import { ProxiesOptions } from '../server/middlewares/proxy.js';

import type { JsPlugin } from '../plugin/index.js';
import type { RustPlugin } from '../plugin/rustPluginResolver.js';
import type { Config } from '../../binding/index.js';
import type cors from '@koa/cors';

export interface UserServerConfig {
  port?: number;
  https?: boolean;
  protocol?: 'http' | 'https';
  hostname?: string;
  // http2?: boolean;
  hmr?: boolean | UserHmrConfig;
  proxy?: Record<string, ProxiesOptions>;
  strictPort?: boolean;
  open?: boolean;
  host?: string;
  cors?: boolean | cors.Options;
}

export type NormalizedServerConfig = Required<
  Omit<UserServerConfig, 'hmr'> & {
    hmr: Required<UserHmrConfig>;
  }
>;

export interface UserHmrConfig {
  /** ignored watch paths of the module graph, entries of this option should be a string regexp  */
  ignores?: string[];
  host?: string;
  port?: number;
}

export interface UserConfig {
  /** current root of this project, default to current working directory */
  root?: string;
  base?: string;
  clearScreen?: boolean;
  envDir?: string;
  envPrefix?: string;
  publicDir?: string;
  /** js plugin(which is a javascript object) and rust plugin(which is string refer to a .farm file or a package) */
  plugins?: (RustPlugin | JsPlugin)[];
  /** config related to compilation */
  compilation?: Config['config'];
  /** config related to dev server */
  server?: UserServerConfig;
}

export interface GlobalFarmCLIOptions {
  '--'?: string[];
  c?: boolean | string;
  config?: string;
  m?: string;
  mode?: 'development' | 'production';
}

export interface FarmCLIServerOptions {
  port?: number;
  open?: boolean;
  https?: boolean;
  hmr?: boolean;
  strictPort?: boolean;
}

export interface FarmCLIBuildOptions {
  outDir?: string;
  sourcemap?: boolean;
  minify?: boolean;
}

export interface FarmCLIPreviewOptions {
  open?: boolean;
  https?: boolean;
  port?: number;
}

export interface FarmCLIOptions
  extends FarmCLIServerOptions,
    FarmCLIBuildOptions,
    FarmCLIPreviewOptions {
  logger?: Logger;
  config?: string;
  configPath?: string;
  clearScreen?: boolean;
}
