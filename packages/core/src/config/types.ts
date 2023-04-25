import type { Config } from '../../binding/index.js';
import { Logger } from '../logger.js';
import type { JsPlugin } from '../plugin/index.js';
import type { RustPlugin } from '../plugin/rustPluginResolver.js';

export interface UserServerConfig {
  port?: number;
  https?: boolean;
  // http2?: boolean;
  hmr?: boolean | UserHmrConfig;
  strictPort?: boolean;
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
  mode?: string;
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

export type FarmCLIOptions = FarmCLIServerOptions &
  FarmCLIBuildOptions & {
    logger?: Logger;
    config?: string;
    configPath?: string;
  };
