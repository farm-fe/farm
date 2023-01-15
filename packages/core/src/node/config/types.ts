import { Config } from '../../../binding/index.js';
import { JsPlugin } from '../plugin/index.js';
import { RustPlugin } from '../plugin/rustPluginResolver.js';

export interface UserServerConfig {
  port: number;
  hmr: boolean | UserHmrConfig;
}

export interface UserHmrConfig {
  /** ignored watch paths of the module graph, entries of this option should be a string regexp  */
  ignores?: string[];
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
