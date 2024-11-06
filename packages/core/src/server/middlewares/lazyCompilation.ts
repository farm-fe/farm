import { existsSync } from 'node:fs';
import { relative } from 'node:path';
import { parse as parseUrl } from 'node:url';

import {
  Server,
  VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX,
  bold,
  cyan,
  formatExecutionTime,
  getDynamicResources,
  green
} from '../../index.js';
import { send } from '../send.js';

import type Connect from 'connect';

const DEFAULT_LAZY_COMPILATION_PATH = '/__lazy_compile';

export function lazyCompilationMiddleware(
  app: Server
): Connect.NextHandleFunction {
  return async function handleLazyCompilationMiddleware(req, res, next) {
    const { resolvedUserConfig, compiler } = app;

    if (!req.url.startsWith(DEFAULT_LAZY_COMPILATION_PATH)) {
      return await next();
    }
    const parsedUrl = parseUrl(req.url, true);
    const paths = (parsedUrl.query.paths as string).split(',');

    const isNodeEnvironment = parsedUrl.query.node;
    const root = compiler.config.root;
    const pathsStr = paths
      .map((p) => {
        if (
          p.startsWith('/') &&
          !p.endsWith(VIRTUAL_FARM_DYNAMIC_IMPORT_SUFFIX) &&
          !existsSync(p)
        ) {
          return p;
        }
        const resolvedPath = compiler.transformModulePath(root, p);
        return relative(root, resolvedPath);
      })
      .join(', ');

    resolvedUserConfig.logger.info(
      `${bold(green('✨Lazy compiling'))} ${bold(cyan(pathsStr))}`,
      true
    );
    const start = performance.now();

    let result;
    try {
      // sync regenerate resources
      result = await compiler.update(paths, true, false, false);
    } catch (e) {
      throw new Error(`Lazy compilation update failed: ${e.message}`);
    }

    if (!result) {
      return next();
    }

    // TODO 取的对象不对 writeToDisk
    // if (isNodeEnvironment || resolvedUserConfig.writeToDisk) {
    if (isNodeEnvironment) {
      compiler.writeResourcesToDisk();
    }

    resolvedUserConfig.logger.info(
      `${bold(green(`✓ Lazy compilation done`))} ${bold(
        cyan(pathsStr)
      )} in ${bold(
        green(
          resolvedUserConfig.logger.formatExecutionTime(
            performance.now() - start
          )
        )
      )}.`
    );

    if (result) {
      const { dynamicResources, dynamicModuleResourcesMap } =
        getDynamicResources(result.dynamicResourcesMap);

      const responseData = {
        dynamicResources,
        dynamicModuleResourcesMap
      };

      const lazyCompilationContent = !isNodeEnvironment
        ? `export default ${JSON.stringify(responseData)}`
        : JSON.stringify(responseData);

      const contentType = !isNodeEnvironment
        ? 'application/javascript'
        : 'application/json';

      send(req, res, lazyCompilationContent, req.url, {
        headers: {
          'Content-Type': contentType,
          'Access-Control-Allow-Origin': '*',
          'Access-Control-Allow-Methods': '*',
          'Access-Control-Allow-Headers': '*'
        }
      });
    } else {
      throw new Error(`Lazy compilation result not found for paths ${paths}`);
    }
  };
}
