import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
import path from 'node:path';

import fse from 'fs-extra';

import {
  getValidPublicPath,
  isArray,
  isNodeEnv,
  normalizePath
} from '../../utils/index.js';
import {
  DEFAULT_DEV_SERVER_OPTIONS,
  FARM_DEFAULT_NAMESPACE,
  ResolvedCompilation,
  ResolvedUserConfig
} from '../index.js';

export function normalizeRuntimeConfig(
  resolvedCompilation: ResolvedCompilation,
  resolvedUserConfig: ResolvedUserConfig,
  isProduction: boolean
) {
  const resolvedRootPath = resolvedCompilation.root;
  const require = createRequire(import.meta.url);
  const hmrClientPluginPath = require.resolve('@farmfe/runtime-plugin-hmr');
  const importMetaPluginPath = require.resolve(
    '@farmfe/runtime-plugin-import-meta'
  );

  resolvedCompilation.runtime = {
    ...(resolvedCompilation.runtime ?? {}),
    path:
      resolvedCompilation.runtime?.path ??
      path.dirname(require.resolve('@farmfe/runtime/package.json')),
    swcHelpersPath:
      resolvedCompilation.runtime?.swcHelpersPath ??
      path.dirname(require.resolve('@swc/helpers/package.json')),
    plugins: resolvedCompilation.runtime?.plugins ?? [],
    namespace: resolvedCompilation.runtime?.namespace,
    isolate:
      resolvedCompilation.runtime?.isolate ??
      resolvedCompilation.output?.targetEnv !== 'browser-esnext'
  };

  resolvedCompilation.runtime.plugins = resolvedCompilation.runtime.plugins.map(
    (plugin) => {
      if (path.isAbsolute(plugin)) return plugin;
      return plugin.startsWith('.')
        ? path.resolve(resolvedRootPath, plugin)
        : require.resolve(plugin);
    }
  );

  if (!resolvedCompilation.runtime.namespace) {
    resolvedCompilation.runtime.namespace = createHash('md5')
      .update(getNamespaceName(resolvedRootPath))
      .digest('hex');
  }

  const isNode = isNodeEnv(resolvedCompilation.output.targetEnv);
  if (
    !isProduction &&
    !isNode &&
    isArray(resolvedCompilation.runtime.plugins) &&
    resolvedUserConfig.server?.hmr &&
    !resolvedCompilation.runtime.plugins.includes(hmrClientPluginPath)
  ) {
    const publicPath = getValidPublicPath(
      resolvedCompilation.output.publicPath
    );
    const serverOptions = resolvedUserConfig.server;
    const defineHmrPath = normalizePath(
      path.join(publicPath, resolvedUserConfig.server.hmr.path)
    );

    resolvedCompilation.runtime.plugins.push(hmrClientPluginPath);

    resolvedCompilation.define.FARM_HMR_PORT = String(
      (serverOptions.hmr.port || undefined) ??
        serverOptions.port ??
        DEFAULT_DEV_SERVER_OPTIONS.port
    );
    resolvedCompilation.define.FARM_HMR_HOST = JSON.stringify(
      resolvedUserConfig.server.hmr.host
    );
    resolvedCompilation.define.FARM_HMR_PROTOCOL = JSON.stringify(
      resolvedUserConfig.server.hmr.protocol
    );
    resolvedCompilation.define.FARM_HMR_PATH = JSON.stringify(defineHmrPath);
  }

  if (
    isArray(resolvedCompilation.runtime.plugins) &&
    !resolvedCompilation.runtime.plugins.includes(importMetaPluginPath)
  ) {
    resolvedCompilation.runtime.plugins.push(importMetaPluginPath);
  }
}

function getNamespaceName(rootPath: string) {
  const packageJsonPath = path.resolve(rootPath, 'package.json');
  if (fse.existsSync(packageJsonPath)) {
    const { name } = JSON.parse(fse.readFileSync(packageJsonPath, 'utf-8'));
    return name || FARM_DEFAULT_NAMESPACE;
  }
  return FARM_DEFAULT_NAMESPACE;
}
