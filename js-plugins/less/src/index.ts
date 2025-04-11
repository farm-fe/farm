import { existsSync } from 'fs';
import path from 'path';
import {
  Compiler,
  JsPlugin,
  UserConfig,
  getAdditionContext
} from '@farmfe/core';
import { createLessResolvePlugin } from './plugin-resolve.js';
import {
  getLessImplementation,
  pluginName,
  throwError,
  tryRead
} from './utils.js';

export type LessPluginOptions = {
  lessOptions?: Less.Options;
  implementation?: string;
  filters?: {
    resolvedPaths?: string[];
    moduleTypes?: string[];
  };
  additionalData?:
    | string
    | ((content?: string, resolvePath?: string) => string | Promise<string>);
};

export default function farmLessPlugin(
  options: LessPluginOptions = {}
): JsPlugin {
  let farmConfig: UserConfig['compilation'];
  let compiler: Compiler;
  const implementation: LessStatic = getLessImplementation(
    options?.implementation
  );

  // @ts-ignore TODO fix it
  const cwd = () => farmConfig.root ?? process.cwd();

  return {
    name: pluginName,
    config(config) {
      if (!config?.compilation?.resolve?.extensions) {
        config.compilation ??= {};
        config.compilation.resolve ??= {};
        config.compilation.resolve.extensions ??= [];
      }

      config.compilation.resolve.extensions = [
        ...new Set(config.compilation.resolve.extensions.concat('less'))
      ];
      return config;
    },
    configResolved: (config) => {
      farmConfig = config.compilation;
      const preprocessorOptions =
        config.compilation?.css?._viteCssOptions?.preprocessorOptions?.less ??
        {};
      options.lessOptions = {
        ...options.lessOptions,
        ...preprocessorOptions
      };
    },
    // @ts-ignore it will be removed in the future
    configureServer() {
      console.warn(
        '[@farmfe/js-plugin-less] Your plugin version is not compatible with the current farm version, please update @farmfe/core to the latest version, otherwise the plugin may not work properly.'
      );
    },
    configureCompiler(c) {
      compiler = c;
    },
    load: {
      filters: {
        resolvedPaths: options.filters?.resolvedPaths ?? ['\\.less$']
      },
      async executor(param) {
        if (param.query.length === 0 && existsSync(param.resolvedPath)) {
          const data = await tryRead(param.resolvedPath);

          return {
            content: data,
            moduleType: 'less'
          };
        }

        return null;
      }
    },
    transform: {
      filters: {
        resolvedPaths: options.filters?.resolvedPaths,
        moduleTypes: options.filters?.moduleTypes ?? ['less']
      },
      async executor(param, ctx) {
        try {
          const isProd = farmConfig.mode === 'production';
          let relData = '';
          const fileRoot = path.dirname(param.resolvedPath);
          const configPaths = options.lessOptions?.paths;
          const additionContext = await getAdditionContext(
            cwd(),
            options,
            param.resolvedPath,
            param.content,
            ctx,
            pluginName
          );
          if (additionContext) {
            relData = `${additionContext}\n${param.content}`;
            //  If the additionalData is a function, it might be return null or undefined, so we need to check it
            if (typeof relData !== 'string') {
              relData = param.content;
            }
          } else {
            relData = param.content;
          }

          const sourceMapEnabled = ctx.sourceMapEnabled(param.moduleId);
          const pluginResolve = createLessResolvePlugin(
            implementation,
            ctx,
            param.resolvedPath
          );

          const { css, map, imports } = await implementation.render(relData, {
            ...(options?.lessOptions ?? {}),
            filename: param.resolvedPath,
            plugins: [pluginResolve, ...(options.lessOptions?.plugins ?? [])],
            sourceMap:
              (options.lessOptions?.sourceMap ?? sourceMapEnabled) && {},
            paths: configPaths ? [fileRoot, ...configPaths] : [fileRoot]
          } as Less.Options);

          if (compiler && imports && !isProd) {
            for (const dep of imports) {
              compiler.addExtraWatchFile(param.moduleId, [
                path.resolve(fileRoot, dep)
              ]);
            }
          }
          return {
            content: css,
            moduleType: 'css',
            sourceMap: map
          };
        } catch (error) {
          throwError('transform', error);
        }
        return {
          content: '',
          moduleType: 'css'
        };
      }
    }
  };
}
