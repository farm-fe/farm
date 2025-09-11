import { SecureServerOptions } from 'node:http2';
import type { ServerOptions as HttpsServerOptions } from 'node:https';
import type { WatchOptions } from 'chokidar';
import type Connect from 'connect';
import type { OutgoingHttpHeaders } from 'http';
import { CompilationMode, PreviewServer, Server } from '../index.js';
import type { RustPlugin } from '../plugin/rust/index.js';
import type { JsPlugin } from '../plugin/type.js';
import { ProxyOptions } from '../server/middlewares/proxy.js';
import type { Config, CssConfig, OutputConfig } from '../types/binding.js';
import type { Logger } from '../utils/index.js';

/**
 * Interface for Hot Module Reload (HMR) configuration options.
 * HMR allows for updating parts of an application without a full reload,
 * improving development efficiency.
 */
export interface HmrOptions {
  /**
   * The protocol to use for the HMR connection.
   *
   * @default ''
   */
  protocol?: string;
  /**
   * The host address for the HMR server.
   *
   * @default 'localhost'
   */
  host?: string;
  /**
   * The port number for the HMR server.
   *
   * @default undefined
   */
  port?: number;
  /**
   * The port number for the HMR client.
   *
   * @default 9000
   */
  clientPort?: number;
  /**
   * The path for the HMR endpoint.
   *
   * @default '/__hmr'
   */
  path?: string;
  /**
   * The timeout in milliseconds for the HMR connection.
   *
   * @default 0
   */
  timeout?: number;
  /**
   * Whether to show an overlay for HMR errors in the browser.
   *
   * @default true
   */
  overlay?: boolean;
  /**
   * The server instance to attach the HMR middleware to.
   *
   * @default null
   */
  server?: Server;
}

export interface ConfigEnv {
  mode: string;
  command: commandType;
  isPreview: boolean;
}

export type commandType = 'dev' | 'build' | 'watch' | 'preview';

export type UserConfigFnPromise = (env: ConfigEnv) => Promise<UserConfig>;
export type UserConfigFn = (env: ConfigEnv) => UserConfig | Promise<UserConfig>;
export type UserConfigFnObject = (env: ConfigEnv) => UserConfig;

export type UserConfigExport =
  | UserConfig
  | Promise<UserConfig>
  | UserConfigFnObject
  | UserConfigFnPromise
  | UserConfigFn;

/**
 * Interface for user server configuration, defining various
 * configuration options for the server.
 */
export interface UserServerConfig {
  /**
   * HTTP headers to be sent with every response.
   *
   * @default {}
   */
  headers?: OutgoingHttpHeaders | undefined;
  /**
   * The port number the server will listen on.
   *
   * @default 9000
   */
  port?: number;
  /**
   * Configuration options for the HTTPS server.
   *
   * @default undefined
   */
  https?: HttpsServerOptions;
  /**
   * The origin address of the server.
   *
   * @default ''
   */
  origin?: string;
  allowedHosts?: string[];
  // http2?: boolean;
  /**
   * Configuration options for Hot Module Replacement (HMR),
   * which can be a boolean or a detailed configuration object.
   */
  hmr?: boolean | HmrOptions;
  /**
   * Proxy configuration for the server, in key-value pair format.
   *
   * @default undefined
   */
  proxy?: Record<string, ProxyOptions>;
  /**
   * Whether to strictly use the specified port. If the port is
   * occupied, an exception will be thrown when this option is set
   * to `true`.
   *
   * @default false
   */
  strictPort?: boolean;
  /**
   * Whether to automatically open the server in the default browser.
   *
   * @default false
   */
  open?: boolean;
  /**
   * The host address the server listens on.
   *
   * @default 'localhost'
   */
  host?: string | boolean;
  /**
   * Whether to enable CORS (Cross-Origin Resource Sharing),
   * which can be a boolean or detailed configuration.
   *
   * @default false
   */
  cors?: boolean | any;
  /**
   * Application type, with optional values of 'spa'
   * (Single Page Application), 'mpa' (Multi-Page Application),
   * or 'custom' (Custom).
   *
   * @default 'spa'
   */
  appType?: 'spa' | 'mpa' | 'custom';
  /**
   * Array of middleware for the development server.
   *
   * @default []
   */
  middlewares?: DevServerMiddleware[];
  /**
   * Whether to run the server in middleware mode.
   *
   * @default false
   */
  middlewareMode?: boolean;
  /**
   * Whether to write files to disk.
   *
   * @default false
   */
  writeToDisk?: boolean;
  /**
   * Configuration options for the preview server.
   */
  preview?: UserPreviewServerConfig;
}

/** Preview server configs */
export interface UserPreviewServerConfig {
  /**
   * HTTP headers to be sent with every response.
   * Set to `false` to disable preview server headers.
   *
   * @default server.headers
   */
  headers?: OutgoingHttpHeaders | false | undefined;
  /**
   * Host to run the preview server on.
   *
   * @default 'localhost'
   */
  host?: string | boolean;
  /**
   * Port to run the preview server on.
   *
   * **NOTE**: If the port is already in use, the preview
   * server will automatically try the next available port.
   * If you want to use a specific port strictly, please
   * set `strictPort` to `true`.
   *
   * @default 1911
   */
  port?: number;
  /**
   * Use the specified port strictly.
   *
   * If the enabled, the preview server will throw an exception
   * if failed to binding on specified port.
   *
   * @default false
   */
  strictPort?: boolean;
  /**
   * Secure server options.
   *
   * Set to `false` to disable https options.
   *
   * @default server.https
   */
  https?: SecureServerOptions;
  /**
   * Specify where the dist directory is located.
   * If not specified, farm will try to resolve
   * the dist directory from `compilation.output.path`.
   * If the path is relative, this will be relative to `root`.
   *
   * @default 'dist'
   */
  distDir?: string;
  /**
   * Open the preview server in the default browser automatically.
   *
   * @default false
   */
  open?: boolean | string;
  /**
   * Enable CORS for preview server.
   *
   * @default false
   */
  cors?: boolean | any;
  /**
   * Proxy options for preview server.
   * Set to `false` to disable proxy.
   *
   * @default server.proxy
   */
  proxy?: Record<string, ProxyOptions>;
  /**
   * Array of middleware for the development server.
   *
   * @default []
   */
  middlewares?: PreviewServerMiddleware[];
}

export type NormalizedServerConfig = Required<
  Omit<UserServerConfig, 'hmr'> & {
    hmr?: HmrOptions;
    protocol?: 'http' | 'https';
    hostname?: { name: string; host: string | undefined };
  }
>;

type InternalConfig = Config['config'] extends undefined
  ? object
  : Required<Config>['config'];

type AvailableUserConfigKeys = Exclude<
  keyof InternalConfig,
  'configFilePath' | 'env' | 'coreLibPath' | 'root'
>;

/**
 * Interface for user configuration, defining various configuration options for the project.
 */
export interface UserConfig {
  /**
   * The root directory of the current project, defaulting to the current working directory.
   *
   * @default cwd
   */
  root?: string;
  /**
   * Whether to clear the screen when starting.
   *
   * @default false
   */
  clearScreen?: boolean;
  /**
   * The mode of the project, such as 'development' or 'production'.
   */
  mode?: string;
  /**
   * The directory where the environment variable files are located.
   */
  envDir?: string;
  /**
   * Whether to enable file watching, or the configuration options for watching.
   */
  watch?: boolean | WatchOptions;
  /**
   * The prefix for environment variables, which can be a single string or an array of strings.
   *
   * @default 'FARM_'
   */
  envPrefix?: string | string[];
  /**
   * The public directory, where files under this dir will always be treated as static assets.
   * Static assets will be served in development server, directly copied to the output directory.
   *
   * @default 'public'
   */
  publicDir?: string;
  /**
   * List of farm plugins, supporting JavaScript plugins, Rust plugins, or arrays of plugins.
   * You can pass null, undefined, or false to disable plugins.
   */
  plugins?: (RustPlugin | JsPlugin | JsPlugin[] | undefined | null | false)[];
  /**
   * List of Vite compatible plugins.
   */
  vitePlugins?: (
    | null
    | undefined
    | object
    | (() => { vitePlugin: any; filters: string[] })
  )[];
  /**
   * Configuration related to compilation.
   */
  compilation?: Pick<InternalConfig, AvailableUserConfigKeys>;
  /**
   * Configuration related to the server.
   */
  server?: UserServerConfig;
  /**
   * Custom logger instance.
   */
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
  command?: commandType;
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
  mode?: string;
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
  configFile?: string | false;
  compilation?: Config['config'];
  mode?: string;
  root?: string;
  server?: UserConfig['server'];
  clearScreen?: boolean;
}

export type DevServerMiddleware = (
  context: Server
) => Connect.NextHandleFunction | undefined;
export type PreviewServerMiddleware = (
  context: PreviewServer
) => Connect.NextHandleFunction | undefined;

export interface Alias {
  find: string;
  replacement: string;
}
export type Format = Exclude<OutputConfig['format'], undefined>;

export type DefaultOptionsType = {
  inlineOptions?: FarmCliOptions;
  configFilePath?: string;
  format?: Format;
  outputPath?: string;
  fileName?: string;
  mode?: CompilationMode;
  showFileSize?: boolean;
};

export type EnvResult = Record<
  `$__farm_regex:(global(This)?\\.)?process\\.env\\.${string}`,
  string
>;

export interface ConfigResult {
  config: UserConfig;
  configFilePath: string;
}
