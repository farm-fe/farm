import { z } from 'zod';
import { fromZodError } from 'zod-validation-error';

import type { UserConfig } from './types.js';

const compilationConfigSchema = z
  .object({
    root: z.string().optional(),
    input: z.record(z.string()).optional(),
    output: z
      .object({
        entryFilename: z.string().optional(),
        filename: z.string().optional(),
        path: z.string().optional(),
        publicPath: z.string().optional(),
        assetsFilename: z.string().optional(),
        targetEnv: z.enum(['browser', 'node']).optional(),
        format: z.enum(['cjs', 'esm']).optional()
      })
      .strict()
      .optional(),
    resolve: z
      .object({
        extensions: z.array(z.string()).optional(),
        alias: z.record(z.string()).optional(),
        mainFields: z.array(z.string()).optional(),
        conditions: z.array(z.string()).optional(),
        symlinks: z.boolean().optional(),
        strictExports: z.boolean().optional()
      })
      .strict()
      .optional(),
    define: z.record(z.string()).optional(),
    external: z.array(z.string()).optional(),
    mode: z.string().optional(),
    watch: z
      .union([
        z.boolean(),
        z.object({
          // TODO watcher config schema
          /* your watcher config schema */
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
        path: z.string().nonempty(),
        plugins: z.array(z.string()).optional(),
        swcHelpersPath: z.string().optional()
      })
      .strict()
      .optional(),
    assets: z
      .object({
        include: z.array(z.string()).optional()
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
            'es2022'
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
        plugins: z.array(z.any()).optional()
      })
      .strict()
      .optional(),
    sourcemap: z
      .union([z.boolean(), z.literal('all'), z.literal('inline')])
      .optional(),
    partialBundling: z
      .object({
        targetConcurrentRequests: z.number().positive().int().optional(),
        targetMinSize: z.number().positive().int().optional(),
        targetMaxSize: z.number().positive().int().optional(),
        groups: z
          .array(
            z.object({
              name: z.string(),
              test: z.array(z.string()),
              groupType: z.enum(['mutable', 'immutable']),
              resourceType: z.enum(['all', 'initial', 'async'])
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
        immutableModulesWeight: z.number().optional()
      })
      .strict()
      .optional(),
    lazyCompilation: z.boolean().optional(),
    treeShaking: z.boolean().optional(),
    minify: z.boolean().optional(),
    record: z.boolean().optional(),
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
          .object({
            indentName: z.string().optional()
          })
          .optional(),
        prefixer: z
          .object({
            targets: z
              .string()
              .or(z.record(z.string()))
              .or(z.array(z.string()))
              .optional()
          })
          .optional()
      })
      .optional(),
    html: z.object({ base: z.string().optional() }).optional()
  })
  .strict();

const FarmConfigSchema = z
  .object({
    root: z.string().optional(),
    base: z.string().optional(),
    clearScreen: z.boolean().optional(),
    configPath: z.string().optional(),
    envDir: z.string().optional(),
    envPrefix: z.union([z.string(), z.array(z.string())]).optional(),
    publicDir: z.string().optional(),
    plugins: z.array(z.any()).optional(),
    vitePlugins: z.array(z.any()).optional(),
    compilation: compilationConfigSchema.optional(),
    server: z
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
        https: z.boolean().optional(),
        cors: z.boolean().optional(),
        proxy: z
          .record(
            z.object({
              target: z.string(),
              changeOrigin: z.boolean(),
              rewrite: z
                .function(z.tuple([z.string(), z.object({})]))
                .optional()
            })
          )
          .optional(),
        strictPort: z.boolean().optional(),
        hmr: z
          .union([
            z.boolean(),
            z
              .object({
                ignores: z.array(z.string()).optional(),
                host: z.string().nonempty().optional(),
                port: z.number().positive().int().optional(),
                watchOptions: z
                  .object({
                    awaitWriteFinish: z.number().positive().int().optional()
                  })
                  .optional()
              })
              .strict()
          ])
          .optional(),
        plugins: z.array(z.any()).optional(),
        writeToDisk: z.boolean().optional()
      })
      .strict()
      .optional()
  })
  .strict();

export function parseUserConfig(config: unknown) {
  try {
    const parsed = FarmConfigSchema.parse(config);
    return parsed as UserConfig;
    // return config as UserConfig;
  } catch (err) {
    const validationError = fromZodError(err);
    // the error now is readable by the user
    throw new Error(
      `${validationError}. \n Please check your configuration file or command line configuration.`
    );
  }
}
