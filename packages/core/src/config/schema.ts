import { z } from 'zod';
import { UserConfig } from './types.js';

const ConfigSchema = z
  .object({
    coreLibPath: z.string().optional(),
    input: z.record(z.string()).optional(),
    output: z
      .object({
        filename: z.string().optional(),
        path: z.string().optional(),
        publicPath: z.string().optional(),
        assetsFilename: z.string().optional(),
        targetEnv: z.enum(['browser', 'node']).optional(),
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
        strictExports: z.boolean().optional(),
      })
      .strict()
      .optional(),
    define: z.record(z.string()).optional(),
    external: z.array(z.string()).optional(),
    mode: z.enum(['development', 'production']).optional(),
    root: z.string().optional(),
    runtime: z
      .object({
        path: z.string().nonempty(),
        plugins: z.array(z.string()).optional(),
        swcHelpersPath: z.string().optional(),
      })
      .strict()
      .optional(),
    assets: z
      .object({
        include: z.array(z.string()).optional(),
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
          ])
          .optional(),
        parser: z.object({
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
              allowReturnOutsideFunction: z.boolean(),
            })
            .strict()
            .optional(),
          tsConfig: z
            .object({
              tsx: z.boolean(),
              decorators: z.boolean(),
              dts: z.boolean(),
              noEarlyErrors: z.boolean(),
            })
            .strict()
            .optional(),
        }),
      })
      .strict()
      .optional(),
    sourcemap: z.union([z.boolean(), z.literal('all')]).optional(),
    partialBundling: z
      .object({
        moduleBuckets: z.array(
          z
            .object({
              name: z.string(),
              test: z.array(z.string()),
            })
            .strict()
        ),
      })
      .strict()
      .optional(),
    lazyCompilation: z.boolean().optional(),
    treeShaking: z.boolean().optional(),
    minify: z.boolean().optional(),
  })
  .strict();

const RustPluginSchema = z.union([
  z.string().nonempty(),
  z.tuple([z.string(), z.record(z.any())]),
]);

const HookExecutor = z.function(
  z.tuple([z.any(), z.any().optional(), z.any().optional()]),
  z.any()
);

const JSPluginSchema = z
  .object({
    name: z.string().nonempty(),
    priority: z.number().optional(),
    config: z.any().optional(),
    resolve: z
      .object({
        filters: z
          .object({
            importers: z.array(z.string()),
            sources: z.array(z.string()),
          })
          .strict(),
        executor: HookExecutor,
      })
      .strict()
      .optional(),
    load: z
      .object({
        filters: z
          .object({
            resolvedPaths: z.array(z.string()),
          })
          .strict(),
        executor: HookExecutor,
      })
      .strict()
      .optional(),
    transform: z
      .object({
        filters: z
          .object({
            resolvedPaths: z.array(z.string()),
          })
          .strict(),
        executor: HookExecutor,
      })
      .strict()
      .optional(),
  })
  .strict();

const UserConfigSchema = z
  .object({
    root: z.string().optional(),
    plugins: z.array(z.union([RustPluginSchema, JSPluginSchema])).optional(),
    compilation: ConfigSchema.optional(),
    server: z
      .object({
        port: z.number().positive().int().optional(),
        https: z.boolean().optional(),
        hmr: z
          .union([
            z.boolean(),
            z
              .object({
                ignores: z.array(z.string()).optional(),
                host: z.string().nonempty().optional(),
                port: z.number().positive().int().optional(),
              })
              .strict(),
          ])
          .optional(),
      })
      .strict()
      .optional(),
  })
  .strict();

export function parseUserConfig(config: unknown) {
  const parsed = UserConfigSchema.parse(config);
  // TODO: parse will only return correct types if tsconfig is set to strict
  return parsed as UserConfig;
}
