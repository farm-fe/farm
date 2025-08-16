import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import path from 'node:path';

import fse from 'fs-extra';

import { bindingPath } from '../../../binding/index.js';
import {
  colors,
  isArray,
  isEmptyObject,
  isNodeEnv,
  normalizePath
} from '../../utils/index.js';
import merge from '../../utils/merge.js';
import {
  CUSTOM_KEYS,
  DEFAULT_COMPILATION_OPTIONS,
  DEFAULT_DEV_SERVER_OPTIONS,
  ENV_DEVELOPMENT,
  ENV_PRODUCTION,
  FARM_DEFAULT_NAMESPACE
} from '../constants.js';
import { CompilationMode, setProcessEnv } from '../env.js';
import {
  EnvResult,
  ResolvedCompilation,
  ResolvedUserConfig
} from '../types.js';

import { normalizeCss } from './normalize-css.js';
import { normalizeExternal } from './normalize-external.js';
import { getValidPublicPath, normalizeOutput } from './normalize-output.js';
import { normalizePersistentCache } from './normalize-persistent-cache.js';
import { normalizeResolve } from './normalize-resolve.js';

/**
 * Normalize user config and transform it to rust compiler compatible config
 *
 *
 * ResolvedUserConfig is a parameter passed to rust Compiler,
 * and ResolvedUserConfig is generated from UserConfig.
 * When UserConfig is different from ResolvedUserConfig,
 * a legal value should be given to the ResolvedUserConfig field here,
 * and converted from UserConfig in the subsequent process.
 *
 * @param config
 * @returns resolved config that parsed to rust compiler
 */
export async function normalizeUserCompilationConfig(
  resolvedUserConfig: ResolvedUserConfig,
  mode: CompilationMode = 'development'
): Promise<ResolvedCompilation> {
  const { compilation, root = process.cwd() } = resolvedUserConfig;

  // resolve root path
  const resolvedRootPath = root;

  resolvedUserConfig.root = resolvedRootPath;

  // if normalize default config, skip check input option
  const inputIndexConfig = await checkCompilationInputValue(resolvedUserConfig);

  const resolvedCompilation: ResolvedCompilation = merge(
    {},
    DEFAULT_COMPILATION_OPTIONS,
    {
      input: inputIndexConfig,
      root: resolvedRootPath
    },
    compilation
  );

  const isProduction = mode === ENV_PRODUCTION;
  const isDevelopment = mode === ENV_DEVELOPMENT;
  resolvedCompilation.mode = resolvedCompilation.mode ?? mode;

  resolvedCompilation.coreLibPath = bindingPath;

  normalizeOutput(resolvedCompilation, isProduction, resolvedUserConfig.logger);
  normalizeExternal(resolvedUserConfig, resolvedCompilation);

  // @ts-ignore do not check type for this internal option
  if (!resolvedCompilation.assets?.publicDir) {
    resolvedCompilation.assets ??= {};

    const userPublicDir = resolvedUserConfig.publicDir
      ? resolvedUserConfig.publicDir
      : path.join(resolvedCompilation.root, 'public');

    if (path.isAbsolute(userPublicDir)) {
      // @ts-ignore do not check type for this internal option
      resolvedCompilation.assets.publicDir = userPublicDir;
    } else {
      // @ts-ignore do not check type for this internal option
      resolvedCompilation.assets.publicDir = path.join(
        resolvedCompilation.root,
        userPublicDir
      );
    }
  }

  resolvedCompilation.define = Object.assign(
    {
      // skip self define
      ['FARM' + '_PROCESS_ENV']: resolvedUserConfig.env
    },
    resolvedCompilation?.define,
    // for node target, we should not define process.env.NODE_ENV
    resolvedCompilation.output?.targetEnv === 'node' ||
      resolvedCompilation.output?.targetEnv === 'library'
      ? {}
      : Object.keys(resolvedUserConfig.env || {}).reduce<EnvResult>(
          (env, key) => {
            env[`$__farm_regex:(global(This)?\\.)?process\\.env\\.${key}`] =
              JSON.stringify(resolvedUserConfig.env[key]);
            return env;
          },
          {} as EnvResult
        )
  );

  const require = createRequire(import.meta.url);
  const hmrClientPluginPath = require.resolve('@farmfe/runtime-plugin-hmr');
  const importMetaPluginPath = require.resolve(
    '@farmfe/runtime-plugin-import-meta'
  );

  resolvedCompilation.runtime = {
    path:
      resolvedCompilation.runtime?.path ??
      path.dirname(require.resolve('@farmfe/runtime/package.json')),
    swcHelpersPath:
      resolvedCompilation.runtime?.swcHelpersPath ??
      path.dirname(require.resolve('@swc/helpers/package.json')),
    plugins: resolvedCompilation.runtime?.plugins ?? [],
    namespace: resolvedCompilation.runtime?.namespace
  };

  resolvedCompilation.runtime.plugins = resolvedCompilation.runtime.plugins.map(
    (plugin) => {
      if (path.isAbsolute(plugin)) return plugin;
      return plugin.startsWith('.')
        ? path.resolve(resolvedRootPath, plugin)
        : require.resolve(plugin);
    }
  );

  if (!resolvedCompilation.runtime.namespace) {
    resolvedCompilation.runtime.namespace = createHash('md5')
      .update(getNamespaceName(resolvedRootPath))
      .digest('hex');
  }

  if (isProduction) {
    resolvedCompilation.lazyCompilation = false;
  } else if (resolvedCompilation.lazyCompilation === undefined) {
    resolvedCompilation.lazyCompilation ??= isDevelopment;
  }

  resolvedCompilation.mode ??= mode;

  setProcessEnv(resolvedCompilation.mode);
  const isNode = isNodeEnv(resolvedCompilation.output.targetEnv);
  if (
    !isProduction &&
    !isNode &&
    isArray(resolvedCompilation.runtime.plugins) &&
    resolvedUserConfig.server?.hmr &&
    !resolvedCompilation.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    const publicPath = getValidPublicPath(
      resolvedCompilation.output.publicPath
    );
    const serverOptions = resolvedUserConfig.server;
    const defineHmrPath = normalizePath(
      path.join(publicPath, resolvedUserConfig.server.hmr.path)
    );

    resolvedCompilation.runtime.plugins.push(hmrClientPluginPath);
    // TODO optimize get hmr logic
    resolvedCompilation.define.FARM_HMR_PORT = String(
      (serverOptions.hmr.port || undefined) ??
        serverOptions.port ??
        DEFAULT_DEV_SERVER_OPTIONS.port
    );
    resolvedCompilation.define.FARM_HMR_HOST = JSON.stringify(
      resolvedUserConfig.server.hmr.host
    );
    resolvedCompilation.define.FARM_HMR_PROTOCOL = JSON.stringify(
      resolvedUserConfig.server.hmr.protocol
    );
    resolvedCompilation.define.FARM_HMR_PATH = JSON.stringify(defineHmrPath);
  }

  if (
    isArray(resolvedCompilation.runtime.plugins) &&
    !resolvedCompilation.runtime.plugins.includes(importMetaPluginPath)
  ) {
    resolvedCompilation.runtime.plugins.push(importMetaPluginPath);
  }

  // we should not deep merge compilation.input
  if (compilation?.input && Object.keys(compilation.input).length > 0) {
    // Add ./ if userConfig.input is relative path without ./
    const input: Record<string, string> = {};

    for (const [key, value] of Object.entries(compilation.input)) {
      if (!value && (value ?? true)) continue;
      if (!path.isAbsolute(value) && !value.startsWith('./')) {
        input[key] = `./${value}`;
      } else {
        input[key] = value;
      }
    }

    resolvedCompilation.input = input;
  }

  if (resolvedCompilation.treeShaking === undefined) {
    resolvedCompilation.treeShaking ??= isProduction;
  }

  if (resolvedCompilation.concatenateModules === undefined) {
    resolvedCompilation.concatenateModules ??= isProduction;
  }

  if (resolvedCompilation.concatenateModules && !isProduction) {
    resolvedUserConfig.logger.warn(
      'concatenateModules option is not supported with development mode, concatenateModules will be disabled'
    );
    resolvedCompilation.concatenateModules = false;
  }

  if (resolvedCompilation.script?.plugins?.length) {
    resolvedUserConfig.logger.info(
      `Swc plugins are configured, note that Farm uses ${colors.yellow(
        'swc_core v35.0.0'
      )}, please make sure the plugin is ${colors.green('compatible')} with swc_core ${colors.yellow(
        'swc_core v35.0.0'
      )}. Otherwise, it may exit unexpectedly.`
    );
  }

  // lazyCompilation should be disabled in production mode
  // so, it only happens in development mode
  // https://github.com/farm-fe/farm/issues/962
  if (resolvedCompilation.treeShaking && resolvedCompilation.lazyCompilation) {
    resolvedUserConfig.logger.error(
      'treeShaking option is not supported in lazyCompilation mode, lazyCompilation will be disabled.'
    );
    resolvedCompilation.lazyCompilation = false;
  }

  if (resolvedCompilation.minify === undefined) {
    resolvedCompilation.minify ??= isProduction;
  }

  if (resolvedCompilation.presetEnv === undefined) {
    resolvedCompilation.presetEnv ??= isProduction;
  }

  // setting the custom configuration
  resolvedCompilation.custom = {
    ...(resolvedCompilation.custom || {}),
    [CUSTOM_KEYS.runtime_isolate]: `${!!resolvedCompilation.runtime.isolate}`
  };

  // Auto enable decorator by default when `script.decorators` is enabled
  if (resolvedCompilation.script?.decorators !== undefined)
    if (resolvedCompilation.script.parser === undefined) {
      resolvedCompilation.script.parser = {
        esConfig: {
          decorators: true
        },
        tsConfig: {
          decorators: true
        }
      };
    } else {
      if (resolvedCompilation.script.parser.esConfig !== undefined)
        resolvedCompilation.script.parser.esConfig.decorators = true;
      else
        resolvedCompilation.script.parser.esConfig = {
          decorators: true
        };
      if (resolvedCompilation.script.parser.tsConfig !== undefined)
        resolvedCompilation.script.parser.tsConfig.decorators = true;
    }

  // normalize persistent cache at last
  await normalizePersistentCache(resolvedCompilation, resolvedUserConfig);
  normalizeResolve(resolvedUserConfig, resolvedCompilation);
  normalizeCss(resolvedUserConfig, resolvedCompilation);

  return resolvedCompilation;
}

export function normalizePublicDir(root: string, publicDir = 'public') {
  const absPublicDirPath = path.isAbsolute(publicDir)
    ? publicDir
    : path.resolve(root, publicDir);

  return absPublicDirPath;
}

export async function checkCompilationInputValue(
  userConfig: ResolvedUserConfig
) {
  const { compilation } = userConfig;
  const targetEnv = compilation?.output?.targetEnv;
  const inputValue = Object.values(compilation?.input || {}).filter(Boolean);
  const isTargetNode = isNodeEnv(targetEnv);
  const defaultHtmlPath = './index.html';
  let inputIndexConfig: {
    index?: string;
  } = {};
  let errorMessage = '';

  // Check if input is specified
  if (!isEmptyObject(compilation?.input) && inputValue.length) {
    inputIndexConfig = compilation?.input;
  } else {
    const rootPath = userConfig?.root ?? '.';
    if (isTargetNode) {
      // If input is not specified, try to find index.js or index.ts
      const entryFiles = ['./index.js', './index.ts'];

      for (const entryFile of entryFiles) {
        try {
          const resolvedPath = path.resolve(rootPath, entryFile);
          if (fse.existsSync(resolvedPath)) {
            inputIndexConfig = {
              index: entryFile
            };
            break;
          }
        } catch (error) {
          errorMessage = error.stack;
        }
      }
    } else {
      try {
        const resolvedHtmlPath = path.resolve(rootPath, defaultHtmlPath);
        if (fse.existsSync(resolvedHtmlPath)) {
          inputIndexConfig = {
            index: defaultHtmlPath
          };
        }
      } catch (error) {
        errorMessage = error.stack;
      }
    }

    // If no index file is found, throw an error
    if (!inputIndexConfig.index) {
      throw new Error(
        `Build failed due to errors: Can not resolve ${
          isTargetNode ? 'index.js or index.ts' : 'index.html'
        }  from ${userConfig.root}. \n${errorMessage}`
      );
    }
  }

  return inputIndexConfig;
}

function getNamespaceName(rootPath: string) {
  const packageJsonPath = path.resolve(rootPath, 'package.json');
  if (fse.existsSync(packageJsonPath)) {
    const { name } = JSON.parse(fse.readFileSync(packageJsonPath, 'utf-8'));
    return name || FARM_DEFAULT_NAMESPACE;
  }
  return FARM_DEFAULT_NAMESPACE;
}
