import { JsPlugin } from '../index.js';
import {
  type RustPlugin,
  getSortedPlugins,
  resolveAsyncPlugins,
  resolveConfigHook,
  resolveConfigResolvedHook,
  resolveFarmPlugins,
  resolveVitePlugins
} from '../plugin/index.js';
import { externalAdapter } from '../plugin/js/external-adapter.js';

import { wrapPluginUpdateModules } from '../plugin/js/utils.js';

import { resolveHostname } from '../utils/http.js';
import { Logger } from '../utils/index.js';
import { __FARM_GLOBAL__ } from './_global.js';
import { CompilationMode, setProcessEnv } from './env.js';

import { ENV_PRODUCTION } from './constants.js';
import { mergeConfig, mergeFarmCliConfig } from './merge-config.js';

import type {
  ConfigEnv,
  FarmCliOptions,
  ResolvedUserConfig,
  UserConfig,
  UserConfigExport,
  UserConfigFnObject,
  commandType
} from './types.js';

import { loadConfigFile } from './load-config-file.js';
import { normalizeUserCompilationConfig } from './normalize-config/index.js';
import { resolveUserConfig } from './resolve-config.js';
import { normalizeDevServerConfig } from './resolve-server.js';

export {
  normalizeUserCompilationConfig,
  resolveUserConfig,
  normalizeDevServerConfig
};

export * from './types.js';
export * from './constants.js';
export * from './env.js';

export function defineFarmConfig(config: UserConfig): UserConfig;
export function defineFarmConfig(
  config: Promise<UserConfig>
): Promise<UserConfig>;
export function defineFarmConfig(
  config: UserConfigFnObject
): UserConfigFnObject;
export function defineFarmConfig(config: UserConfigExport): UserConfigExport;
export function defineFarmConfig(config: UserConfigExport): UserConfigExport {
  return config;
}

const COMMANDS = {
  START: 'start',
  BUILD: 'build',
  WATCH: 'watch',
  PREVIEW: 'preview',
  CLEAN: 'clean'
} as const;

/**
 * Resolve and load user config from the specified path
 * @param configPath
 */
export async function resolveConfig(
  inlineOptions: FarmCliOptions & UserConfig,
  command: commandType,
  defaultMode: CompilationMode = 'development'
): Promise<ResolvedUserConfig> {
  const mode = inlineOptions.mode || defaultMode;
  const isNodeEnvSet = !!process.env.NODE_ENV;
  inlineOptions.mode = mode;

  if (!isNodeEnvSet) {
    setProcessEnv(defaultMode);
  }

  const configEnv: ConfigEnv = {
    mode,
    command,
    isPreview: command === COMMANDS.PREVIEW
  };

  let configFilePath;

  const loadedUserConfig = await loadConfigFile(
    inlineOptions,
    configEnv,
    defaultMode
  );

  let userConfig: UserConfig = mergeFarmCliConfig(
    inlineOptions,
    {},
    defaultMode
  );

  const transformInlineConfig = userConfig;

  if (loadedUserConfig) {
    configFilePath = loadedUserConfig.configFilePath;
    userConfig = mergeConfig(userConfig, loadedUserConfig.config);
  }

  const { jsPlugins, vitePluginAdapters } = await resolvePlugins(
    userConfig,
    defaultMode
  );
  const sortFarmJsPlugins = getSortedPlugins([
    ...jsPlugins,
    ...vitePluginAdapters
  ]);

  const config = await resolveConfigHook(
    userConfig,
    configEnv,
    sortFarmJsPlugins
  );

  // may be user push plugin when config hooks
  const allPlugins = await resolvePlugins(config, defaultMode);
  const farmJsPlugins = getSortedPlugins([
    ...allPlugins.jsPlugins,
    ...vitePluginAdapters,
    externalAdapter()
  ]);

  const resolvedUserConfig = await handleResolveConfig(
    configFilePath,
    config,
    farmJsPlugins,
    allPlugins.rustPlugins,
    transformInlineConfig,
    command,
    defaultMode
  );

  await resolveConfigResolvedHook(resolvedUserConfig, sortFarmJsPlugins); // Fix: Await the Promise<void> and pass the resolved value to the function.

  return resolvedUserConfig;
}

async function handleResolveConfig(
  configFilePath: string,
  userConfig: UserConfig,
  sortFarmJsPlugins: JsPlugin[],
  rustPlugins: RustPlugin[],
  transformInlineConfig: UserConfig,
  command: commandType,
  mode: CompilationMode
): Promise<ResolvedUserConfig> {
  // define logger when resolvedConfigHook
  const logger = new Logger({
    customLogger: userConfig?.customLogger,
    allowClearScreen: userConfig?.clearScreen
  });

  const resolvedUserConfig = await resolveUserConfig(
    userConfig,
    configFilePath
  );

  resolvedUserConfig.logger = logger;

  // farm handles server attributes in resolveConfig.
  // On the one hand, farm can be used in node and server needs
  // to be enabled. Lazy loading mode is enabled in node environment.
  resolvedUserConfig.server = normalizeDevServerConfig(resolvedUserConfig);

  resolvedUserConfig.compilation = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    mode
  );

  Object.assign(resolvedUserConfig, {
    root: resolvedUserConfig.compilation.root,
    jsPlugins: sortFarmJsPlugins,
    rustPlugins: rustPlugins,
    command,
    isProduction: resolvedUserConfig.compilation.mode === ENV_PRODUCTION,
    transformInlineConfig
  });

  await handleLazyCompilation(
    resolvedUserConfig,
    command as keyof typeof COMMANDS
  );

  return resolvedUserConfig;
}

async function handleLazyCompilation(
  config: ResolvedUserConfig,
  command: keyof typeof COMMANDS
) {
  const commandHandlers = {
    [COMMANDS.START]: async (config: ResolvedUserConfig) => {
      if (
        config.compilation.lazyCompilation &&
        typeof config.server?.host === 'string'
      ) {
        await setLazyCompilationDefine(config);
      }
    },
    [COMMANDS.WATCH]: async (config: ResolvedUserConfig) => {
      if (config.compilation?.lazyCompilation) {
        await setLazyCompilationDefine(config);
      }
    }
  };

  const handler = commandHandlers[command as keyof typeof commandHandlers];
  if (handler) {
    await handler(config);
  }
}

export async function resolvePlugins(
  userConfig: UserConfig,
  mode: CompilationMode
) {
  const [farmPlugins, vitePluginAdapters] = await Promise.all([
    resolveFarmPlugins(userConfig),
    resolveVitePlugins(userConfig, mode)
  ]);

  const resolvePluginsResult = {
    jsPlugins: farmPlugins.jsPlugins.map(wrapPluginUpdateModules),
    vitePlugins: (userConfig?.vitePlugins ?? []).filter(Boolean),
    rustPlugins: farmPlugins.rustPlugins,
    vitePluginAdapters
  };

  return resolvePluginsResult;
}

export async function resolveAndFilterAsyncPlugins(
  plugins: JsPlugin[] = []
): Promise<JsPlugin[]> {
  return (await resolveAsyncPlugins(plugins)).filter(Boolean);
}

async function setLazyCompilationDefine(
  resolvedUserConfig: ResolvedUserConfig
) {
  const hostname = await resolveHostname(resolvedUserConfig.server.host);
  resolvedUserConfig.compilation.define = {
    ...(resolvedUserConfig.compilation.define ?? {}),
    FARM_LAZY_COMPILE_SERVER_URL: `${
      resolvedUserConfig.server.protocol || 'http'
    }://${hostname.host || 'localhost'}:${resolvedUserConfig.server.port}`
  };
}
