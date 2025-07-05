import path from 'node:path';

import { traceDependencies } from '../utils/trace-dependencies.js';
import { ENV_DEVELOPMENT, ENV_PRODUCTION } from './constants.js';
import { getExistsEnvFiles, loadEnv } from './env.js';
import { normalizePublicDir } from './normalize-config/index.js';
import { DefaultOptionsType, ResolvedUserConfig, UserConfig } from './types.js';

export function createDefaultConfig(options: DefaultOptionsType): UserConfig {
  const { inlineOptions, mode, format, outputPath, fileName, configFilePath } =
    options;

  return {
    root: path.resolve(inlineOptions.root ?? '.'),
    compilation: {
      input: {
        [fileName]: configFilePath
      },
      output: {
        entryFilename: '[entryName]',
        path: outputPath,
        format,
        targetEnv: 'library'
      },
      mode,
      external: [
        ...(process.env.FARM_CONFIG_FULL_BUNDLE
          ? []
          : ['!^(\\./|\\.\\./|[A-Za-z]:\\\\|/).*']),
        '^@farmfe/core$'
      ],
      partialBundling: {
        enforceResources: [
          {
            name: fileName,
            test: ['.+']
          }
        ]
      },
      sourcemap: false,
      treeShaking: false,
      minify: false,
      presetEnv: false,
      lazyCompilation: false,
      persistentCache: false,
      progress: false
    }
  };
}

export async function resolveUserConfig(
  userConfig: UserConfig,
  configFilePath?: string | undefined
): Promise<ResolvedUserConfig> {
  const resolvedUserConfig = {
    ...userConfig,
    envMode: userConfig.mode
  } as ResolvedUserConfig;

  // set internal config
  if (configFilePath) {
    const dependencies = await traceDependencies(configFilePath);
    resolvedUserConfig.configFileDependencies = dependencies.sort();
    resolvedUserConfig.configFilePath = configFilePath;
  }

  const resolvedRootPath = resolvedUserConfig.root;
  const resolvedEnvPath = resolvedUserConfig.envDir ?? resolvedRootPath;

  const userEnv = loadEnv(
    resolvedUserConfig.envMode,
    resolvedEnvPath,
    resolvedUserConfig.envPrefix
  );
  const existsEnvFiles = getExistsEnvFiles(
    resolvedUserConfig.envMode,
    resolvedEnvPath
  );

  resolvedUserConfig.envFiles = [
    ...(Array.isArray(resolvedUserConfig.envFiles)
      ? resolvedUserConfig.envFiles
      : []),
    ...existsEnvFiles
  ];

  resolvedUserConfig.env = {
    ...userEnv,
    NODE_ENV: userConfig.compilation.mode,
    BASE_URL: userConfig.compilation?.output?.publicPath ?? '/',
    mode: userConfig.mode,
    DEV: userConfig.compilation.mode === ENV_DEVELOPMENT,
    PROD: userConfig.compilation.mode === ENV_PRODUCTION
  };

  resolvedUserConfig.publicDir = normalizePublicDir(
    resolvedRootPath,
    userConfig.publicDir
  );

  return resolvedUserConfig;
}
