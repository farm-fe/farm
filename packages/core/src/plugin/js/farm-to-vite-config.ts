import type { UserConfig as ViteUserConfig } from 'vite';
import type { UserConfig } from '../../config/types.js';
import {
  deleteUndefinedPropertyDeeply,
  throwIncompatibleError
} from './utils.js';
import merge from 'lodash.merge';
import { Logger } from '../../index.js';

export function farmUserConfigToViteConfig(config: UserConfig): ViteUserConfig {
  const vitePlugins = config.vitePlugins.map((plugin) => {
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
      open: config.server?.open
      // other options are not supported in farm
    },
    // eslint-disable-next-line @typescript-eslint/ban-ts-comment
    // @ts-ignore ignore this error
    isProduction: config.compilation?.mode === 'production',
    css: {
      devSourcemap: false
    },
    build: {
      outDir: config.compilation?.output?.path,
      sourcemap,
      minify: config.compilation?.minify,
      cssMinify: config.compilation?.minify,
      ssr: config.compilation?.output?.targetEnv === 'node',
      rollupOptions: {
        output: {
          assetFileNames: config.compilation?.output?.assetsFilename,
          entryFileNames: config.compilation?.output?.entryFilename,
          chunkFileNames: config.compilation?.output?.filename
        }
      }
      // other options are not supported in farm
    }
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

      if (key === 'then' || key === 'length' || key === 'constructor') {
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
        // these fields are always undefined in farm
        // they are only used for compatibility
        'legacy',
        'optimizeDeps',
        'ssr',
        'logLevel',
        'experimental'
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
                resolveKey === 'constructor'
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
            'origin'
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
                serverKey === 'constructor'
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
          const allowedCssKeys = ['devSourcemap'];

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
                cssKey === 'constructor'
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
            'rollupOptions'
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
                buildKey === 'constructor'
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
              logger.warn(
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
  pluginName: string
): UserConfig {
  const farmConfig: UserConfig = {};

  if (config.root) {
    farmConfig.root = config.root;
  }
  if (config.base) {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    if (!farmConfig.compilation.output) {
      farmConfig.compilation.output = {};
    }
    farmConfig.compilation.output.publicPath = config.base;
  }
  if (config.publicDir) {
    farmConfig.publicDir = config.publicDir;
  }
  if (config.mode === 'development' || config.mode === 'production') {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    farmConfig.compilation.mode = config.mode;
  }
  if (config.define) {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    farmConfig.compilation.define = config.define;
  }
  if (config.resolve) {
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    if (!farmConfig.compilation.resolve) {
      farmConfig.compilation.resolve = {};
    }
    if (config.resolve.alias) {
      if (!Array.isArray(config.resolve.alias)) {
        farmConfig.compilation.resolve.alias = config.resolve.alias as Record<
          string,
          any
        >;
      } else {
        // TODO support array alias
        console.warn(
          `[vite-plugin] ${pluginName}: farm do not support array 'resolve.alias', it will be ignored. you should transform it to farm's alias manually for now.`
        );
      }
    }

    farmConfig.compilation.resolve.extensions = config.resolve.extensions;
    farmConfig.compilation.resolve.mainFields = config.resolve.mainFields;
    farmConfig.compilation.resolve.conditions = config.resolve.conditions;
    farmConfig.compilation.resolve.symlinks =
      config.resolve.preserveSymlinks != true;
  }

  if (config.server) {
    if (!farmConfig.server) {
      farmConfig.server = {};
    }
    farmConfig.server.hmr = config.server.hmr;
    farmConfig.server.port = config.server.port;

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
    if (!farmConfig.compilation) {
      farmConfig.compilation = {};
    }
    if (!farmConfig.compilation.output) {
      farmConfig.compilation.output = {};
    }
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
