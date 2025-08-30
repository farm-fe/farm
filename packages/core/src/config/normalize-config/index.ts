import path from 'node:path';

import fse from 'fs-extra';

import { bindingPath } from '../../../binding/index.js';
import { colors, isEmptyObject, isNodeEnv } from '../../utils/index.js';
import merge from '../../utils/merge.js';
import {
  DEFAULT_COMPILATION_OPTIONS,
  ENV_DEVELOPMENT,
  ENV_PRODUCTION
} from '../constants.js';
import { CompilationMode, setProcessEnv } from '../env.js';
import {
  EnvResult,
  ResolvedCompilation,
  ResolvedUserConfig
} from '../types.js';

import { normalizeCss } from './normalize-css.js';
import { normalizeExternal } from './normalize-external.js';
import { normalizeOutput } from './normalize-output.js';
import { normalizePersistentCache } from './normalize-persistent-cache.js';
import { normalizeResolve } from './normalize-resolve.js';
import { normalizeRuntimeConfig } from './normalize-runtime.js';

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

  if (isProduction) {
    resolvedCompilation.lazyCompilation = false;
  } else if (resolvedCompilation.lazyCompilation === undefined) {
    resolvedCompilation.lazyCompilation ??= isDevelopment;
  }

  resolvedCompilation.mode ??= mode;

  setProcessEnv(resolvedCompilation.mode);

  normalizeRuntimeConfig(resolvedCompilation, resolvedUserConfig, isProduction);

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
