import path from 'path';
import fs from 'fs';
import { mergeObjects } from '../../index.js';
import { JsPlugin } from '../type.js';
import * as querystring from 'querystring';
import {
  convertEnforceToPriority,
  customParseQueryString,
  getContentValue,
  guessIdLoader,
  isObject,
  isString
  // transformQuery,
  // isObject,
  // isString
} from './utils.js';

// export function adaptorVitePlugin<UserOptions = Record<string, never>>(
export function adaptorVitePlugin(rawPlugins: any) {
  if (!rawPlugins?.length) return;
  const plugins = rawPlugins.map((rawPlugin: any) => {
    const farmPlugin: JsPlugin = {
      name: rawPlugin?.name,
      priority: convertEnforceToPriority(rawPlugin?.enforce)
    };

    if (rawPlugin?.handleHotUpdate) {
      // farmPlugin.updateModules = {
      //   executor(result) {
      //     const ctx = {
      //       file: result.paths[0][0],
      //     };
      //     rawPlugin.handleHotUpdate(ctx);
      //   },
      // };
    }

    if (rawPlugin?.config || rawPlugin?.configResolved) {
      farmPlugin.config = function (config: any) {
        const resolveConfig = rawPlugin.config(config);
        const res = mergeObjects(resolveConfig, config);
        delete res.define.__VUE_OPTIONS_API__;
        delete res.define.__VUE_PROD_DEVTOOLS__;
        if (rawPlugin.config) {
          return res;
        }

        if (rawPlugin.configResolved) {
          rawPlugin.configResolved(res);
        }
      };
    }

    if (rawPlugin?.configureServer) {
      farmPlugin.configDevServer = function (server: any) {
        rawPlugin.configureServer(server);
      };
    }

    if (rawPlugin.buildStart) {
      const _buildStart = rawPlugin.buildStart;
      farmPlugin.buildStart = {
        async executor(_, context) {
          await _buildStart.call(context!);
        }
      } as JsPlugin['buildStart'];
    }

    if (rawPlugin.resolveId) {
      const _resolveId = rawPlugin.resolveId;
      farmPlugin.resolve = {
        filters: { sources: ['.*'], importers: ['.*'] },
        async executor(params: any) {
          const resolvedIdPath = path.resolve(
            process.cwd(),
            params.importer?.relativePath ?? ''
          );

          const resolveIdResult = await _resolveId(resolvedIdPath ?? null);
          if (isString(resolveIdResult)) {
            return {
              resolvedPath: resolveIdResult,
              query: customParseQueryString(resolveIdResult),
              sideEffects: false,
              external: false,
              meta: {}
            };
          } else if (isObject(resolveIdResult) as any) {
            return {
              resolvedPath: resolveIdResult?.id,
              query: customParseQueryString(resolveIdResult!.id),
              sideEffects: false,
              external: resolveIdResult?.external,
              meta: {}
            };
          }
          return null;
        }
      } as unknown as JsPlugin['resolve'];
    }

    if (rawPlugin?.load) {
      const _load = rawPlugin.load;
      farmPlugin.load = {
        filters: {
          resolvedPaths: ['.vue$']
        },
        async executor(
          params
          // ctx,
          // hookContext,
        ): Promise<any | null> {
          if (
            rawPlugin.loadInclude &&
            !rawPlugin.loadInclude(params.resolvedPath)
          ) {
            return null;
          }
          const qw = Object.fromEntries(params.query);
          console.log(qw);

          const { vue } = qw;
          console.log(vue);

          if (vue) {
            const loader = guessIdLoader(params.resolvedPath);
            const shouldLoadInclude =
              rawPlugin.loadInclude &&
              rawPlugin.loadInclude(params.resolvedPath);

            const langMap = new Map([
              ['css', 'style'],
              ['scss', 'style'],
              ['less', 'style']
            ]);
            const queryString = `${querystring.stringify(
              Object.fromEntries(params.query)
            )}&type=${langMap.get(params.query[0][1]) ?? 'script'}`;

            const resolvedPathWithQuery = `${params.resolvedPath}?${queryString}`;

            const content: any = await _load(resolvedPathWithQuery, {
              ssr: false
            });

            const loadFarmResult: any = {
              content: getContentValue(content),
              moduleType: loader
            };
            if (shouldLoadInclude) {
              return loadFarmResult;
            }

            return loadFarmResult;
          }
          const resolvedPath = params.resolvedPath;
          let source = '';
          try {
            source = await fs.promises.readFile(resolvedPath, 'utf-8');
          } catch (err) {
            console.log('报错了');
          }
          return {
            content: source,
            moduleType: 'vue'
          };
        }
      } as JsPlugin['load'];
    }

    // if (rawPlugin.transform) {
    //   const _transform = rawPlugin.transform;
    //   farmPlugin.transform = {
    //     filters: { resolvedPaths: [".*$"], moduleTypes: [".*$"] },
    //     async executor(
    //       params: any,
    //     ) {
    //       if (params.query.length) {
    //         transformQuery(params);
    //       }

    //       if (
    //         rawPlugin.transformInclude &&
    //         !rawPlugin.transformInclude(params.resolvedPath)
    //       ) {
    //         return null;
    //       }

    //       const loader = params.moduleType ??
    //         guessIdLoader(params.resolvedPath);

    //       const shouldTransformInclude = rawPlugin.transformInclude &&
    //         rawPlugin.transformInclude(params.resolvedPath);
    //       const resource: any = await _transform(
    //         params.content,
    //         params.resolvedPath,
    //         { ssr: false },
    //       );

    //       if (resource && typeof resource !== "string") {
    //         const transformFarmResult: any = {
    //           content: getContentValue(resource),
    //           moduleType: loader,
    //           sourceMap: JSON.stringify(resource.map),
    //         };
    //         if (shouldTransformInclude) {
    //           return transformFarmResult;
    //         }

    //         return transformFarmResult;
    //       }
    //     },
    //   } as JsPlugin["transform"];
    // }

    return farmPlugin;
  });

  return plugins;
}
