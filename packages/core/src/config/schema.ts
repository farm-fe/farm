import http from 'node:http';
import { SecureServerOptions } from 'node:http2';

import { z } from 'zod';
import { fromZodError } from 'zod-validation-error';

import { ILogger } from '../utils/logger.js';
import type { UserConfig } from './types.js';

const TARGET_ENV = {
  BROWSER: 'browser',
  NODE: 'node',
  NODE_LEGACY: 'node-legacy',
  NODE_NEXT: 'node-next',
  NODE16: 'node16',
  BROWSER_LEGACY: 'browser-legacy',
  BROWSER_ESNEXT: 'browser-esnext',
  BROWSER_ES2015: 'browser-es2015',
  BROWSER_ES2017: 'browser-es2017',
  LIBRARY: 'library',
  LIBRARY_BROWSER: 'library-browser',
  LIBRARY_NODE: 'library-node'
} as const;

const baseRewriteSchema = z.union([
  z.record(z.string(), z.string()),
  z
    .function()
    .args(z.string(), z.any())
    .returns(z.union([z.string(), z.promise(z.string())]))
]);

const stringRewriteSchema = baseRewriteSchema;
const functionRewriteSchema = baseRewriteSchema;

const pathFilterSchema = z.union([
  z.string(),
  z.array(z.string()),
  z
    .function()
    .args(z.string(), z.instanceof(http.IncomingMessage))
    .returns(z.boolean())
]);

const pathRewriteSchema = baseRewriteSchema;

const outputSchema = z
  .object({
    entryFilename: z.string().optional(),
    filename: z.string().optional(),
    path: z.string().optional(),
    publicPath: z.string().optional(),
    assetsFilename: z.string().optional(),
    targetEnv: z
      .custom<(typeof TARGET_ENV)[keyof typeof TARGET_ENV]>((val) => {
        return Object.values(TARGET_ENV).includes(val);
      })
      .optional(),
    format: z.enum(['cjs', 'esm']).optional()
  })
  .strict()
  .optional();

const serverSchema = z
  .object({
    headers: z.record(z.string()).optional(),
    port: z.number().positive().int().optional(),
    host: z
      .union([
        z.string().regex(/^\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}$/),
        z.literal('localhost'),
        z.boolean()
      ])
      .optional(),
    open: z.boolean().optional(),
    https: z.custom<SecureServerOptions>(),
    cors: z.boolean().optional(),
    spa: z.boolean().optional(),
    proxy: z
      .record(
        z
          .object({
            target: z.string(),
            changeOrigin: z.boolean().optional(),
            agent: z.any().optional(),
            secure: z.boolean().optional(),
            logs: z.any().optional(),
            pathRewrite: pathRewriteSchema.optional(),
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
      .optional(),
    strictPort: z.boolean().optional(),
    hmr: z
      .union([
        z.boolean(),
        z
          .object({
            protocol: z.string().optional(),
            host: z.union([z.string().min(1), z.boolean()]).optional(),
            port: z.number().positive().int().optional(),
            path: z.string().optional(),
            watchOptions: z
              .object({
                awaitWriteFinish: z.number().positive().int().optional()
              })
              .optional(),
            overlay: z.boolean().optional()
          })
          .strict()
      ])
      .optional(),
    middlewares: z.array(z.any()).optional(),
    writeToDisk: z.boolean().optional()
  })
  .strict();

const aliasItemSchema = z.object({
  find: z.union([z.string(), z.instanceof(RegExp)]),
  replacement: z.string(),
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
    watch: z
      .union([
        z.boolean(),
        z.object({
          // TODO watcher config schema
          ignored: z.array(z.string()).optional(),
          watchOptions: z
            .object({
              awaitWriteFinish: z.number().positive().int().optional()
            })
            .optional()
        })
      ])
      .optional(),
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
        target: z
          .enum([
            'es3',
            'es5',
            'es2015',
            'es2016',
            'es2017',
            'es2018',
            'es2019',
            'es2020',
            'es2021',
            'es2022',
            'esnext'
          ])
          .optional(),
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
        nativeTopLevelAwait: z.boolean().optional()
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
              resourceType: z.enum(['all', 'initial', 'async']).optional(),
              enforce: z.boolean().optional()
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
          mode: z
            .union([
              z.literal('minify-module'),
              z.literal('minify-resource-pot')
            ])
            .optional()
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
    custom: z.record(z.string(), z.string()).optional()
  })
  .strict();

const FarmConfigSchema = z
  .object({
    root: z.string().optional(),
    clearScreen: z.boolean().optional(),
    configPath: z.string().optional(),
    envDir: z.string().optional(),
    timeUnit: z.union([z.literal('ms'), z.literal('s')]).optional(),
    envPrefix: z.union([z.string(), z.array(z.string())]).optional(),
    publicDir: z.string().optional(),
    plugins: z.array(z.any()).optional(),
    vitePlugins: z.array(z.any()).optional(),
    compilation: compilationConfigSchema.optional(),
    mode: z.string().optional(),
    watch: z.boolean().optional(),
    server: serverSchema.optional(),
    // TODO ANY type
    customLogger: z.any().optional()
  })
  .strict();

export function parseUserConfig(config: UserConfig): UserConfig {
  try {
    const parsed = FarmConfigSchema.parse(config);
    // TODO type not need `as UserConfig`
    return parsed as UserConfig;
  } catch (err) {
    const validationError = fromZodError(err);
    throw new Error(
      `${validationError.toString()}. \n Please check your configuration file or command line configuration.`
    );
  }
}
