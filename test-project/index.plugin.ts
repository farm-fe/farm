import { JsPlugin } from '@farmfe/core';

export default function NestPlugin(): JsPlugin {
  return {
    name: 'NestPlugin',
    config: (config) => {
      const mode =
        config.compilation.mode ?? process.env.NODE_ENV ?? 'development';
      const isDev = mode === 'development';
      const compilation = config.compilation ?? {};

      const script = compilation.script ?? { plugins: [] };
      return {
        compilation: {
          script: {
            plugins: script.plugins,
            target: script.target ?? 'es2019',
            parser: {
              tsConfig: {
                decorators: script.parser?.tsConfig?.decorators ?? true,
                dts: script.parser?.tsConfig?.dts ?? false,
                noEarlyErrors: script.parser?.tsConfig?.noEarlyErrors ?? false,
                tsx: script.parser?.tsConfig?.tsx ?? false,
              },
            },
            decorators: {
              legacyDecorator: script.decorators?.legacyDecorator ?? true,
              decoratorMetadata: script.decorators?.decoratorMetadata ?? true,
              decoratorVersion:
                script.decorators?.decoratorVersion ?? '2021-12',
              includes: [],
              excludes: ['node_modules/**/*'],
            },
          },
          presetEnv: compilation.presetEnv ?? !isDev,
          minify: compilation.minify ?? !isDev,
          output: {
            format: compilation.output?.format ?? 'esm',
            targetEnv: compilation.output?.targetEnv ?? 'node',
            entryFilename:
              compilation.output?.entryFilename ?? '[entryName].js',
            filename: compilation.output?.filename ?? '[name].[hash].mjs',
          },
        },
      };
    },
  };
}
