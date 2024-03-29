import type { UserConfig as ViteUserConfig } from 'vite';
import type { UserConfig } from '../../config/types.js';
import {
  deleteUndefinedPropertyDeeply,
  throwIncompatibleError
} from './utils.js';
import merge from '../../utils/merge.js';
import { Logger } from '../../index.js';
import { VITE_DEFAULT_ASSETS } from './constants.js';

export function farmUserConfigToViteConfig(config: UserConfig): ViteUserConfig {
  const vitePlugins = config.vitePlugins.filter(Boolean).map((plugin) => {
    if (typeof plugin === 'function') {
      return plugin().vitePlugin;
    } else {
      return plugin;
    }
  });

  let sourcemap = true;

  if (config.compilation?.sourcemap !== undefined) {
    sourcemap = Boolean(config.compilation?.sourcemap);
  }

  const viteConfig: ViteUserConfig = {
    root: config.root,
    base: config.compilation?.output?.publicPath ?? '/',
    publicDir: config.publicDir ?? 'public',
    mode: config.compilation?.mode,
    define: config.compilation?.define,
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore ignore this error
    command: config.compilation?.mode === 'production' ? 'build' : 'serve',
    resolve: {
      alias: config.compilation?.resolve?.alias,
      extensions: config.compilation?.resolve?.extensions,
      mainFields: config.compilation?.resolve?.mainFields,
      conditions: config.compilation?.resolve?.conditions,
      preserveSymlinks: config.compilation?.resolve?.symlinks === false
    },
    plugins: vitePlugins,
    server: {
      // eslint-disable-next-line @typescript-eslint/ban-ts-comment
      // @ts-ignore ignore error
      hmr: config.server?.hmr,
      port: config.server?.port,
      host: config.server?.host,
      strictPort: config.server?.strictPort,
      https: config.server?.https,
      proxy: config.server?.proxy as any,
      open: config.server?.open,
      watch:
        typeof config.server?.hmr === 'object'
          ? config.server.hmr?.watchOptions ?? {}
          : {}
      // other options are not supported in farm
    },
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore ignore this error
    isProduction: config.compilation?.mode === 'production',
    css: config.compilation?.css?._viteCssOptions ?? {},
    build: {
      outDir: config.compilation?.output?.path,
      sourcemap,
      minify:
        config.compilation?.minify !== undefined
          ? Boolean(config.compilation?.minify)
          : undefined,
      cssMinify:
        config.compilation?.minify !== undefined
          ? Boolean(config.compilation?.minify)
          : undefined,
      ssr: config.compilation?.output?.targetEnv === 'node',
      rollupOptions: {
        output: {
          assetFileNames: config.compilation?.output?.assetsFilename,
          entryFileNames: config.compilation?.output?.entryFilename,
          chunkFileNames: config.compilation?.output?.filename
        }
      }
      // other options are not supported in farm
    },
    // TODO make it configurable
    cacheDir: 'node_modules/.farm/cache',
    envDir: config.envDir,
    assetsInclude: [
      ...VITE_DEFAULT_ASSETS,
      ...(config.compilation?.assets?.include ?? [])
    ]
  };

  return viteConfig;
}

export function proxyViteConfig(
  viteConfig: ViteUserConfig,
  pluginName: string,
  logger: Logger
): ViteUserConfig {
  return new Proxy(viteConfig, {
    get(target, key) {
      if (typeof key !== 'string') {
        return target[key as unknown as keyof typeof target];
      }

      if (
        key === 'then' ||
        key === 'length' ||
        key === 'constructor' ||
        key === 'prototype'
      ) {
        return (target as Record<string, any>)[key];
      }

      const allowedKeys = [
        'root',
        'base',
        'publicDir',
        'mode',
        'define',
        'command',
        'resolve',
        'plugins',
        'server',
        'isProduction',
        'css',
        'build',
        'logger',
        'cacheDir',
        'envDir',
        'assetsInclude',
        // these fields are always undefined in farm
        // they are only used for compatibility
        'legacy',
        'optimizeDeps',
        'ssr',
        'logLevel',
        'experimental',
        'test',
        'clearScreen',
        'customLogger'
      ];

      if (allowedKeys.includes(String(key))) {
        if (key === 'resolve') {
          const allowedResolveKeys = [
            'alias',
            'extensions',
            'mainFields',
            'conditions',
            'preserveSymlinks',
            // farm do not set any thing for dedupe, it should always be undefined
            'dedupe'
          ];

          return new Proxy(target.resolve || {}, {
            get(resolveTarget, resolveKey) {
              if (typeof resolveKey !== 'string') {
                return target[resolveKey as unknown as keyof typeof target];
              }

              if (allowedResolveKeys.includes(String(resolveKey))) {
                return resolveTarget[resolveKey as keyof typeof resolveTarget];
              }

              if (
                resolveKey === 'then' ||
                resolveKey === 'length' ||
                resolveKey === 'constructor' ||
                resolveKey === 'prototype'
              ) {
                return (target as Record<string, any>)[key];
              }

              throw throwIncompatibleError(
                pluginName,
                'viteConfig.resolve',
                allowedResolveKeys,
                resolveKey
              );
            }
          });
        } else if (key === 'server') {
          const allowedServerKeys = [
            'hmr',
            'port',
            'host',
            'strictPort',
            'https',
            'proxy',
            'open',
            'origin',
            'watch'
          ];

          return new Proxy(target.server || {}, {
            get(serverTarget, serverKey) {
              if (typeof serverKey !== 'string') {
                return target[serverKey as unknown as keyof typeof target];
              }

              if (allowedServerKeys.includes(String(serverKey))) {
                return serverTarget[serverKey as keyof typeof serverTarget];
              }

              if (
                serverKey === 'then' ||
                serverKey === 'length' ||
                serverKey === 'constructor' ||
                serverKey === 'prototype'
              ) {
                return (target as Record<string, any>)[key];
              }

              throw throwIncompatibleError(
                pluginName,
                'viteConfig.server',
                allowedServerKeys,
                serverKey
              );
            }
          });
        } else if (key === 'css') {
          const allowedCssKeys = [
            'devSourcemap',
            'transformer',
            'modules',
            'postcss',
            'preprocessorOptions'
          ];

          return new Proxy(target.css || {}, {
            get(cssTarget, cssKey) {
              if (typeof cssKey !== 'string') {
                return target[cssKey as unknown as keyof typeof target];
              }

              if (allowedCssKeys.includes(String(cssKey))) {
                return cssTarget[cssKey as keyof typeof cssTarget];
              }

              if (
                cssKey === 'then' ||
                cssKey === 'length' ||
                cssKey === 'constructor' ||
                cssKey === 'prototype'
              ) {
                return (target as Record<string, any>)[key];
              }

              throw throwIncompatibleError(
                pluginName,
                'viteConfig.css',
                allowedCssKeys,
                cssKey
              );
            }
          });
        } else if (key === 'build') {
          const allowedBuildKeys = [
            'outDir',
            'sourcemap',
            'minify',
            'cssMinify',
            'ssr',
            'watch',
            'rollupOptions',
            'assetsDir'
          ];

          return new Proxy(target.build || {}, {
            get(buildTarget, buildKey) {
              if (typeof buildKey !== 'string') {
                return target[buildKey as unknown as keyof typeof target];
              }

              if (allowedBuildKeys.includes(String(buildKey))) {
                return buildTarget[buildKey as keyof typeof buildTarget];
              }
              if (
                buildKey === 'then' ||
                buildKey === 'length' ||
                buildKey === 'constructor' ||
                buildKey === 'prototype'
              ) {
                return (target as Record<string, any>)[key];
              }

              throw throwIncompatibleError(
                pluginName,
                'viteConfig.build',
                allowedBuildKeys,
                buildKey
              );
            }
          });
        } else if (key === 'optimizeDeps') {
          return new Proxy(target.optimizeDeps || {}, {
            get(_, optimizeDepsKey) {
              logger.warnOnce(
                `[vite-plugin] ${pluginName}: config "optimizeDeps" is not needed in farm, all of its options will be ignored. Current ignored option is: "${String(
                  optimizeDepsKey
                )}"`
              );

              if (optimizeDepsKey === 'esbuildOptions') {
                return {};
              }
              return undefined;
            }
          });
        } else if (key === 'logger') {
          return logger;
        } else if (key === 'assetsInclude') {
          return (filename: string) => {
            return (
              (viteConfig.assetsInclude as string[])?.some((r) => {
                return new RegExp(r).test(filename);
              }) ?? false
            );
          };
        }

        return target[key as keyof typeof target];
      }

      throw throwIncompatibleError(pluginName, 'viteConfig', allowedKeys, key);
    }
  });
}

export function viteConfigToFarmConfig(
  config: ViteUserConfig,
  origFarmConfig: UserConfig,
  _pluginName: string
): UserConfig {
  const farmConfig: UserConfig = {
    compilation: {}
  };

  if (config.root) {
    farmConfig.root = config.root;
  }
  if (config?.css) {
    farmConfig.compilation.css ??= {};
    farmConfig.compilation.css._viteCssOptions = config.css;
  }

  if (config.base) {
    farmConfig.compilation.output ??= {};
    farmConfig.compilation.output.publicPath = config.base;
  }
  if (config.publicDir) {
    farmConfig.publicDir = config.publicDir;
  }
  if (config.mode === 'development' || config.mode === 'production') {
    farmConfig.compilation.mode = config.mode;
  }
  if (config.define) {
    farmConfig.compilation.define = config.define;
  }
  if (config.resolve) {
    farmConfig.compilation.resolve ??= {};

    if (config.resolve.alias) {
      if (!Array.isArray(config.resolve.alias)) {
        farmConfig.compilation.resolve.alias = config.resolve.alias as Record<
          string,
          any
        >;
      } else {
        if (!farmConfig.compilation.resolve.alias) {
          farmConfig.compilation.resolve.alias = {};
        }

        const farmRegexPrefix = '$__farm_regex:';

        for (const { find, replacement } of config.resolve.alias) {
          if (find instanceof RegExp) {
            const key = farmRegexPrefix + find.source;
            farmConfig.compilation.resolve.alias[key] = replacement;
          } else {
            farmConfig.compilation.resolve.alias[find] = replacement;
          }
        }
      }
    }

    farmConfig.compilation.resolve.extensions = config.resolve.extensions;
    farmConfig.compilation.resolve.mainFields = config.resolve.mainFields;
    farmConfig.compilation.resolve.conditions = config.resolve.conditions;
    farmConfig.compilation.resolve.symlinks =
      config.resolve.preserveSymlinks != true;
  }

  if (config.server) {
    farmConfig.server ??= {};
    farmConfig.server.hmr = config.server.hmr;
    farmConfig.server.port = config.server.port;

    if (config.server.watch) {
      if (
        farmConfig.server?.hmr === true ||
        farmConfig.server?.hmr === undefined
      ) {
        farmConfig.server.hmr = {
          ...(typeof origFarmConfig?.server?.hmr === 'object'
            ? origFarmConfig.server.hmr
            : {}),
          watchOptions: config.server.watch
        };
      }
    }

    if (typeof config.server.host === 'string') {
      farmConfig.server.host = config.server.host;
    }

    farmConfig.server.strictPort = config.server.strictPort;
    farmConfig.server.https =
      typeof config.server.https === 'boolean'
        ? undefined
        : config.server.https;
    farmConfig.server.proxy = config.server.proxy as any;
    farmConfig.server.open = Boolean(config.server.open);
  }

  if (config.build) {
    farmConfig.compilation.output ??= {};
    farmConfig.compilation.output.path = config.build.outDir;

    if (
      config.build?.sourcemap !== undefined &&
      origFarmConfig.compilation?.sourcemap === undefined
    ) {
      farmConfig.compilation.sourcemap = Boolean(config.build.sourcemap);
    }

    if (config.build?.minify !== undefined) {
      farmConfig.compilation.minify = Boolean(config.build.minify);
    }

    if (
      config.build.ssr !== undefined &&
      origFarmConfig.compilation?.lazyCompilation === undefined
    ) {
      farmConfig.compilation.lazyCompilation = !config.build.ssr;
    }

    if (config.build.rollupOptions?.output !== undefined) {
      if (!Array.isArray(config.build.rollupOptions.output)) {
        const keys = ['assetFileNames', 'entryFilename', 'filename'];

        for (const k of keys) {
          /* eslint-disable @typescript-eslint/ban-ts-comment */
          // @ts-ignore type is correct
          farmConfig.compilation.output[k] =
            // @ts-ignore type is correct
            config.build.rollupOptions.output[k];
        }
      }
    }
  }

  deleteUndefinedPropertyDeeply(farmConfig);

  return merge({}, origFarmConfig, farmConfig);
}
