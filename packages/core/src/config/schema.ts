import http from 'node:http';

import { z } from 'zod';
import { fromZodError } from 'zod-validation-error';

import { Logger } from '../utils/logger.js';

import type { OutgoingHttpHeaders, SecureServerOptions } from 'node:http2';
import type { UserConfig } from './types.js';

enum TargetEnv {
  BROWSER = 'browser',
  NODE = 'node',
  NODE_LEGACY = 'node-legacy',
  NODE_NEXT = 'node-next',
  NODE16 = 'node16',
  BROWSER_LEGACY = 'browser-legacy',
  BROWSER_ESNEXT = 'browser-esnext',
  BROWSER_ES2015 = 'browser-es2015',
  BROWSER_ES2017 = 'browser-es2017',
  LIBRARY = 'library'
  // LIBRARY_BROWSER = 'library-browser',
  // LIBRARY_NODE = 'library-node'
}

enum ECMAVersion {
  ES3 = 'es3',
  ES5 = 'es5',
  ES2015 = 'es2015',
  ES2016 = 'es2016',
  ES2017 = 'es2017',
  ES2018 = 'es2018',
  ES2019 = 'es2019',
  ES2020 = 'es2020',
  ES2021 = 'es2021',
  ES2022 = 'es2022',
  ESNext = 'esnext'
}

const baseRewriteSchema = z.union([
  z.record(z.string(), z.string()),
  z
    .function()
    .args(z.string(), z.any())
    .returns(z.union([z.string(), z.promise(z.string())]))
]);

const pathFilterSchema = z.union([
  z.string(),
  z.array(z.string()),
  z
    .function()
    .args(z.string(), z.instanceof(http.IncomingMessage))
    .returns(z.boolean())
]);

const outputSchema = z
  .object({
    entryFilename: z.string().optional(),
    filename: z.string().optional(),
    path: z.string().optional(),
    publicPath: z.string().optional(),
    assetsFilename: z.string().optional(),
    targetEnv: z.nativeEnum(TargetEnv).optional(),
    format: z.enum(['cjs', 'esm']).optional(),
    showFileSize: z.boolean().optional()
  })
  .strict()
  .optional();

const proxySchema = z
  .record(
    z
      .object({
        target: z.string(),
        changeOrigin: z.boolean().optional(),
        agent: z.any().optional(),
        secure: z.boolean().optional(),
        logs: z.any().optional(),
        pathRewrite: baseRewriteSchema.optional(),
        pathFilter: pathFilterSchema.optional(),
        headers: z.record(z.string()).optional(),
        on: z
          .object({
            proxyReq: z
              .function()
              .args(z.any(), z.any(), z.any())
              .returns(z.void())
              .optional(),
            proxyRes: z
              .function()
              .args(z.any(), z.any(), z.any())
              .returns(z.void())
              .optional(),
            error: z
              .function()
              .args(z.instanceof(Error), z.any(), z.any())
              .returns(z.void())
              .optional()
          })
          .optional()
      })
      .passthrough()
  )
  .optional();

const previewServerSchema = z
  .object({
    headers: z.union([z.custom<OutgoingHttpHeaders>(), z.boolean()]).optional(),
    host: z
      .union([
        z.string().regex(/^\d{1,3}\.\d{1,3}$/),
        z.literal('localhost'),
        z.boolean()
      ])
      .optional(),
    port: z.number().positive().int().optional(),
    strictPort: z.boolean().optional(),
    https: z.custom<SecureServerOptions>(),
    distDir: z.string().optional(),
    open: z.boolean().optional(),
    proxy: proxySchema,
    cors: z.union([z.boolean(), z.any()]).optional()
  })
  .strict();

const WatchOptionsSchema = z.object({
  persistent: z.boolean().optional(),
  ignored: z
    .union([
      z.string(),
      z.array(z.string()),
      z.instanceof(RegExp),
      z.array(z.instanceof(RegExp)),
      z.function(z.tuple([z.string()]), z.boolean())
    ])
    .optional(),
  ignoreInitial: z.boolean().optional(),
  followSymlinks: z.boolean().optional(),
  cwd: z.string().optional(),
  useFsEvents: z.boolean().optional(),
  disableGlobbing: z.boolean().optional(),
  usePolling: z.boolean().optional(),
  interval: z.number().positive().int().optional(),
  binaryInterval: z.number().positive().int().optional(),
  alwaysStat: z.boolean().optional(),
  depth: z.number().positive().int().optional(),
  awaitWriteFinish: z
    .object({
      stabilityThreshold: z.number().positive().int().optional(),
      pollInterval: z.number().positive().int().optional()
    })
    .optional(),
  atomic: z.union([z.boolean(), z.number().positive().int()]).optional(),
  ignorePermissionErrors: z.boolean().optional()
});

const serverSchema = z
  .object({
    headers: z.record(z.string()).optional(),
    port: z.number().positive().int().optional(),
    https: z.custom<SecureServerOptions>(),
    origin: z.string().optional(),
    hmr: z
      .union([
        z.boolean(),
        z
          .object({
            protocol: z.string().optional(),
            host: z.union([z.string().min(1), z.boolean()]).optional(),
            port: z.number().positive().int().optional(),
            path: z.string().optional(),
            overlay: z.boolean().optional()
          })
          .strict()
      ])
      .optional(),
    proxy: proxySchema,
    strictPort: z.boolean().optional(),
    open: z.boolean().optional(),
    host: z
      .union([
        z.string().regex(/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/),
        z.literal('localhost'),
        z.boolean()
      ])
      .optional(),
    cors: z.boolean().optional(),
    appType: z.enum(['spa', 'mpa', 'custom']).optional(),
    middlewares: z.array(z.any()).optional(),
    middlewareMode: z.boolean().optional(),
    writeToDisk: z.boolean().optional(),
    preview: previewServerSchema.optional()
  })
  .strict();

const aliasItemSchema = z.object({
  find: z.union([z.string(), z.instanceof(RegExp)]),
  replacement: z.string(),
  // TODO add customResolver schema
  customResolver: z
    .union([z.function(), z.object({ resolve: z.function() })])
    .optional()
});

const aliasSchema = z.union([z.record(z.string()), z.array(aliasItemSchema)]);

const compilationConfigSchema = z
  .object({
    root: z.string().optional(),
    input: z.record(z.string()).optional(),
    output: outputSchema,
    resolve: z
      .object({
        extensions: z.array(z.string()).optional(),
        alias: aliasSchema.optional(),
        mainFields: z.array(z.string()).optional(),
        conditions: z.array(z.string()).optional(),
        symlinks: z.boolean().optional(),
        strictExports: z.boolean().optional(),
        autoExternalFailedResolve: z.boolean().optional(),
        dedupe: z.array(z.string()).optional()
      })
      .strict()
      .optional(),
    define: z.record(z.any()).optional(),
    external: z
      .array(z.string().or(z.record(z.string(), z.string())))
      .optional(),
    externalNodeBuiltins: z
      .union([z.boolean(), z.array(z.string())])
      .optional(),
    mode: z.string().optional(),

    coreLibPath: z.string().optional(),
    runtime: z
      .object({
        path: z.string().optional(),
        plugins: z.array(z.string()).optional(),
        swcHelpersPath: z.string().optional(),
        isolate: z.boolean().optional()
      })
      .strict()
      .optional(),
    assets: z
      .object({
        include: z.array(z.string()).optional(),
        publicDir: z.string().optional(),
        mode: z.enum(['browser', 'node']).optional()
      })
      .strict()
      .optional(),
    script: z
      .object({
        target: z.nativeEnum(ECMAVersion).optional(),
        parser: z
          .object({
            esConfig: z
              .object({
                jsx: z.boolean().optional(),
                fnBind: z.boolean(),
                decorators: z.boolean(),
                decoratorsBeforeExport: z.boolean(),
                exportDefaultFrom: z.boolean(),
                importAssertions: z.boolean(),
                privateInObject: z.boolean(),
                allowSuperOutsideMethod: z.boolean(),
                allowReturnOutsideFunction: z.boolean()
              })
              .strict()
              .optional(),
            tsConfig: z
              .object({
                tsx: z.boolean().optional(),
                decorators: z.boolean().optional(),
                dts: z.boolean().optional(),
                noEarlyErrors: z.boolean().optional()
              })
              .strict()
              .optional()
          })
          .optional(),
        decorators: z
          .object({
            legacyDecorator: z.boolean().optional(),
            decoratorMetadata: z.boolean().optional(),
            decoratorVersion: z
              .union([z.literal('2021-12'), z.literal('2022-03')])
              .optional(),
            includes: z.array(z.string()).optional(),
            excludes: z.array(z.string()).optional()
          })
          .optional(),
        plugins: z.array(z.any()).optional(),
        nativeTopLevelAwait: z.boolean().optional(),
        importNotUsedAsValues: z
          .union([
            z.literal('remove'),
            z.literal('preserve'),
            z.object({
              preserve: z.array(z.string()).optional()
            })
          ])
          .optional()
      })
      .strict()
      .optional(),
    sourcemap: z
      .union([
        z.boolean(),
        z.literal('all'),
        z.literal('inline'),
        z.literal('all-inline')
      ])
      .optional(),
    partialBundling: z
      .object({
        targetConcurrentRequests: z.number().positive().int().optional(),
        targetMinSize: z.number().nonnegative().int().optional(),
        targetMaxSize: z.number().nonnegative().int().optional(),
        groups: z
          .array(
            z.object({
              name: z.string(),
              test: z.array(z.string()),
              groupType: z.enum(['mutable', 'immutable']).optional(),
              resourceType: z.enum(['all', 'initial', 'async']).optional()
            })
          )
          .optional(),
        enforceResources: z
          .array(
            z
              .object({
                name: z.string(),
                test: z.array(z.string())
              })
              .strict()
          )
          .optional(),
        enforceTargetConcurrentRequests: z.boolean().optional(),
        enforceTargetMinSize: z.boolean().optional(),
        immutableModules: z.array(z.string()).optional(),
        immutableModulesWeight: z.number().optional(),
        enforce: z.boolean().optional()
      })
      .strict()
      .optional(),
    lazyCompilation: z.boolean().optional(),
    treeShaking: z.boolean().optional(),
    minify: z
      .union([
        z.boolean(),
        z.object({
          compress: z.union([z.any(), z.boolean()]).optional(),
          mangle: z.union([z.any(), z.boolean()]).optional(),
          exclude: z.array(z.string()).optional(),
          include: z.array(z.string()).optional(),
          mangleExports: z.boolean().optional()
        })
      ])
      .optional(),
    record: z.boolean().optional(),
    progress: z.boolean().optional(),
    presetEnv: z
      .union([
        z.boolean(),
        z.object({
          include: z.array(z.string()).optional(),
          exclude: z.array(z.string()).optional(),
          options: z.any().optional(),
          assumptions: z.any().optional()
        })
      ])
      .optional(),
    css: z
      .object({
        modules: z
          .union([
            z.null(),
            z.object({
              indentName: z.string().optional(),
              localsConversion: z.string().optional(),
              paths: z.array(z.string()).optional()
            })
          ])

          .optional(),
        prefixer: z
          .union([
            z.null(),
            z.object({
              targets: z
                .string()
                .or(z.record(z.string()))
                .or(z.array(z.string()))
                .optional()
            })
          ])
          .optional()
      })
      .optional(),
    html: z.object({ base: z.string().optional() }).optional(),
    persistentCache: z.union([
      z.boolean(),
      z
        .object({
          namespace: z.string().optional(),
          cacheDir: z.string().optional(),
          buildDependencies: z.array(z.string()).optional(),
          moduleCacheKeyStrategy: z
            .object({
              timestamp: z.boolean().optional(),
              hash: z.boolean().optional()
            })
            .optional(),
          envs: z.record(z.string(), z.string()).optional(),
          globalBuiltinCacheKeyStrategy: z
            .object({
              env: z.boolean().optional(),
              define: z.boolean().optional(),
              buildDependencies: z.boolean().optional(),
              lockfile: z.boolean().optional(),
              packageJson: z.boolean().optional()
            })
            .optional()
        })
        .optional()
    ]),
    comments: z.union([z.boolean(), z.literal('license')]).optional(),
    custom: z.record(z.string(), z.string()).optional(),
    concatenateModules: z.boolean().optional()
  })
  .strict();

const FarmConfigSchema = z
  .object({
    root: z.string().optional(),
    mode: z.string().optional(),
    configPath: z.string().optional(),
    clearScreen: z.boolean().optional(),
    customLogger: z.instanceof(Logger).optional(),
    envDir: z.string().optional(),
    envPrefix: z.union([z.string(), z.array(z.string())]).optional(),
    publicDir: z.string().optional(),
    watch: z.union([z.boolean(), WatchOptionsSchema]).optional(),
    compilation: compilationConfigSchema.optional(),
    server: serverSchema.optional(),
    plugins: z.array(z.any()).optional(),
    vitePlugins: z.array(z.any()).optional()
  })
  .strict();

export function parseUserConfig(config: UserConfig): UserConfig {
  try {
    const parsed = FarmConfigSchema.parse(config);
    return parsed as UserConfig;
  } catch (err) {
    const validationError = fromZodError(err);
    throw new Error(
      `${validationError.toString()}. \n Please check your configuration file or command line configuration.`
    );
  }
}
