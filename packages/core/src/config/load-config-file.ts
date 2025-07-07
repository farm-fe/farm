import path from 'node:path';
import { pathToFileURL } from 'node:url';

import fse from 'fs-extra';

import { ModuleFormat } from '../types/binding.js';
import { convertErrorMessage } from '../utils/error.js';
import { isObject, isWindows } from '../utils/share.js';
import { DEFAULT_CONFIG_NAMES, ENV_PRODUCTION } from './constants.js';
import { CompilationMode } from './env.js';
import { normalizeUserCompilationConfig } from './normalize-config/index.js';
import { createDefaultConfig, resolveUserConfig } from './resolve-config.js';
import { parseUserConfig } from './schema.js';
import {
  ConfigEnv,
  ConfigResult,
  DefaultOptionsType,
  FarmCliOptions,
  ResolvedUserConfig,
  UserConfig
} from './types.js';

async function getConfigFilePath(
  configRootPath: string
): Promise<string | undefined> {
  const stat = await fse.stat(configRootPath);
  if (!stat.isDirectory()) {
    return undefined;
  }

  for (const name of DEFAULT_CONFIG_NAMES) {
    const resolvedPath = path.join(configRootPath, name);
    try {
      const fileStat = await fse.stat(resolvedPath);
      if (fileStat.isFile()) {
        return resolvedPath;
      }
    } catch {}
  }

  return undefined;
}

export async function resolveConfigFilePath(
  configFile: string | undefined,
  root: string,
  configRootPath: string
): Promise<string | undefined> {
  if (configFile) {
    return path.resolve(root, configFile);
  } else {
    return await getConfigFilePath(configRootPath);
  }
}

/**
 * Load config file from the specified path and return the config and config file path
 * @param configPath the config path, could be a directory or a file
 * @param logger custom logger
 * @returns loaded config and config file path
 */
export async function loadConfigFile(
  inlineOptions: FarmCliOptions & UserConfig,
  configEnv: ConfigEnv,
  mode: CompilationMode = 'development'
): Promise<ConfigResult | undefined> {
  const { root = '.', configFile } = inlineOptions;
  const configRootPath = path.resolve(root);
  let resolvedConfigFilePath: string | undefined;
  try {
    resolvedConfigFilePath = await resolveConfigFilePath(
      configFile,
      root,
      configRootPath
    );

    const config = await readConfigFile(
      inlineOptions,
      resolvedConfigFilePath,
      configEnv,
      mode
    );
    return {
      config: config && parseUserConfig(config),
      configFilePath: resolvedConfigFilePath
    };
  } catch (error) {
    // In this place, the original use of throw caused emit to the outermost catch
    // callback, causing the code not to execute. If the internal catch compiler's own
    // throw error can solve this problem, it will not continue to affect the execution of
    // external code. We just need to return the default config.
    const errorMessage = convertErrorMessage(error);
    const stackTrace =
      error.code === 'GenericFailure' ? '' : `\n${error.stack}`;
    if (mode === ENV_PRODUCTION) {
      throw new Error(
        `Failed to load farm config file: ${errorMessage} \n${stackTrace}`
      );
    }
    const potentialSolution =
      'Potential solutions: \n1. Try set `FARM_CONFIG_FORMAT=cjs`(default to esm)\n2. Try set `FARM_CONFIG_FULL_BUNDLE=1`';
    throw new Error(
      `Failed to load farm config file: ${errorMessage}. \n ${potentialSolution} \n ${error.stack}`
    );
    // throw new Error(
    //   `Failed to load farm config file: ${errorMessage}. \n ${potentialSolution}`
    //   // `Failed to load farm config file: ${errorMessage}.`,
    // );
  }
}

const FORMAT_TO_EXT: Record<ModuleFormat, string> = {
  cjs: 'cjs',
  esm: 'mjs'
};

const FORMAT_FROM_EXT: Record<string, ModuleFormat> = {
  cjs: 'cjs',
  mjs: 'esm',
  cts: 'cjs',
  mts: 'esm',
  js: 'esm'
};

function getFilePath(outputPath: string, fileName: string): string {
  return isWindows
    ? pathToFileURL(path.join(outputPath, fileName)).toString()
    : path.join(outputPath, fileName);
}

function getFormat(configFilePath: string): ModuleFormat {
  return process.env.FARM_CONFIG_FORMAT === 'cjs'
    ? 'cjs'
    : process.env.FARM_CONFIG_FORMAT === 'esm'
      ? 'esm'
      : (FORMAT_FROM_EXT[path.extname(configFilePath).slice(1)] ?? 'esm');
}

export async function readConfigFile(
  inlineOptions: FarmCliOptions,
  configFilePath: string,
  configEnv: ConfigEnv,
  mode: CompilationMode = 'development'
): Promise<UserConfig | undefined> {
  if (!fse.existsSync(configFilePath)) return;

  const format = getFormat(configFilePath);

  const Compiler = (await import('../compiler/index.js')).Compiler;

  const outputPath = path.join(
    path.dirname(configFilePath),
    'node_modules',
    '.farm'
  );

  const fileName = `farm.config.bundle-${Date.now()}-${Math.random()
    .toString(16)
    .split('.')
    .join('')}.${FORMAT_TO_EXT[format]}`;

  const normalizedConfig = await resolveDefaultUserConfig({
    inlineOptions,
    configFilePath,
    format,
    outputPath,
    fileName,
    mode
  });
  // disable show file size
  normalizedConfig.output.showFileSize = false;

  const replaceDirnamePlugin = await import(
    '@farmfe/plugin-replace-dirname'
  ).then((mod) => mod.default);

  const compiler = new Compiler({
    compilation: {
      ...normalizedConfig,
      output: { ...normalizedConfig.output, showFileSize: false }
    },
    jsPlugins: [],
    rustPlugins: [[replaceDirnamePlugin, '{}']]
  });

  const FARM_PROFILE = process.env.FARM_PROFILE;
  // disable FARM_PROFILE in farm_config
  if (FARM_PROFILE) {
    process.env.FARM_PROFILE = '';
  }

  try {
    await compiler.compile();

    if (FARM_PROFILE) {
      process.env.FARM_PROFILE = FARM_PROFILE;
    }

    compiler.writeResourcesToDisk();

    const filePath = getFilePath(outputPath, fileName);

    // Change to vm.module of node or loaders as far as it is stable
    const userConfig = (await import(filePath as string)).default;
    try {
      await fse.unlink(filePath);
      // remove parent dir if empty
      const isEmpty = (await fse.readdir(outputPath)).length === 0;
      if (isEmpty) {
        fse.rmSync(outputPath);
      }
    } catch {
      /** do nothing */
    }

    const config = await (typeof userConfig === 'function'
      ? userConfig(configEnv)
      : userConfig);

    if (!isObject(config)) {
      throw new Error(`config must export or return an object.`);
    }

    config.root ??= inlineOptions.root;

    return config;
  } finally {
    await fse.unlink(getFilePath(outputPath, fileName)).catch(() => {});
  }
}

async function resolveDefaultUserConfig(options: DefaultOptionsType) {
  const defaultConfig: UserConfig = createDefaultConfig(options);

  const resolvedUserConfig: ResolvedUserConfig = await resolveUserConfig(
    defaultConfig,
    undefined
  );

  const normalizedConfig = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    options.mode
  );

  return normalizedConfig;
}
