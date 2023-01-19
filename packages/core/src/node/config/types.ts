import type { Config } from '../../../binding/index.js';
import type { JsPlugin } from '../plugin/index.js';
import type { RustPlugin } from '../plugin/rustPluginResolver.js';

export interface UserServerConfig {
  port?: number;
  https?: boolean;
  // http2?: boolean;
  writeToDisk?: boolean;
  hmr?: boolean | UserHmrConfig;
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
