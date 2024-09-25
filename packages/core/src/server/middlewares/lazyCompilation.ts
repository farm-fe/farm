import { existsSync } from 'node:fs';
import { relative } from 'node:path';
import url from 'node:url';

import {
  Resource,
  VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX,
  bold,
  cyan,
  green
} from '../../index.js';
import { send } from '../send.js';

const DEFAULT_LAZY_COMPILATION_PATH = '/__lazy_compile';

export function lazyCompilationMiddleware(app: any) {
  return async function handleLazyCompilationMiddleware(
    req: any,
    res: any,
    next: any
  ) {
    const { resolvedUserConfig, compiler } = app;

    if (!req.url.startsWith(DEFAULT_LAZY_COMPILATION_PATH)) {
      return next();
    }
    const parsedUrl = url.parse(req.url, true);
    const paths = (parsedUrl.query.paths as string).split(',');
    const queryNode = parsedUrl.query.node;
    const pathsStr = paths
      .map((p) => {
        if (
          p.startsWith('/') &&
          !p.endsWith(VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX) &&
          !existsSync(p)
        ) {
          return p;
        }
        const resolvedPath = compiler.transformModulePath(
          compiler.config.config.root,
          p
        );
        return relative(compiler.config.config.root, resolvedPath);
      })
      .join(', ');

    console.info(`Lazy compiling ${bold(cyan(pathsStr))}`);
    const start = performance.now();

    let result;
    try {
      // TODO 我想知道能不能把时间算法 单独写成一个 middleware
      // sync regenerate resources
      result = await compiler.update(paths, true, false, false);
    } catch (e) {
      console.log(e);
    }

    if (!result) {
      return;
    }

    if (queryNode || resolvedUserConfig.writeToDisk) {
      compiler.writeResourcesToDisk();
    }

    console.info(
      `${bold(green(`✓`))} Lazy compilation done ${bold(
        cyan(pathsStr)
      )} in ${bold(green(`${performance.now() - start}ms`))}.`
    );

    if (result) {
      let dynamicResourcesMap: Record<string, Resource[]> = null;

      if (result.dynamicResourcesMap) {
        for (const [key, value] of Object.entries(result.dynamicResourcesMap)) {
          if (!dynamicResourcesMap) {
            dynamicResourcesMap = {} as Record<string, Resource[]>;
          }

          // @ts-ignore
          dynamicResourcesMap[key] = value.map((r) => ({
            path: r[0],
            type: r[1] as 'script' | 'link'
          }));
        }
      }

      const returnObj = `{
          "dynamicResourcesMap": ${JSON.stringify(dynamicResourcesMap)}
        }`;
      const code = !queryNode ? `export default ${returnObj}` : returnObj;

      const contentType = !queryNode
        ? 'application/javascript'
        : 'application/json';

      const lazyCompilationHeaders = {
        'Content-Type': contentType,
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Methods': '*',
        'Access-Control-Allow-Headers': '*'
      };
      send(req, res, code, req.url, {
        headers: lazyCompilationHeaders
      });
    } else {
      throw new Error(`Lazy compilation result not found for paths ${paths}`);
    }
  };
}
