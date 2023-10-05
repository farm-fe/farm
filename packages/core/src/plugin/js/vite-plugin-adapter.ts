import { JsPlugin, CompilationContext } from '../type.js';
import {
  convertEnforceToPriority,
  customParseQueryString,
  formatId,
  formatLoadModuleType,
  formatTransformModuleType,
  getContentValue,
  isObject,
  isString,
  encodeStr,
  decodeStr
} from './utils.js';
import { DevServer, UserConfig } from '../../index.js';

// only use types from vite and we do not install vite as a dependency
import type { Plugin, UserConfig as ViteUserConfig } from 'vite';
import type { PluginContext, ResolveIdResult } from 'rollup';
import path from 'path';
import {
  PluginLoadHookParam,
  PluginLoadHookResult,
  PluginResolveHookParam,
  PluginResolveHookResult,
  PluginTransformHookParam,
  PluginTransformHookResult
} from '../../../binding/index.js';
import merge from 'lodash.merge';

/// turn a vite plugin to farm js plugin
export class VitePluginAdapter implements JsPlugin {
  name = 'to-be-override';
  priority = 0;

  private _rawPlugin: Plugin;
  private _farmConfig: UserConfig;
  private _viteConfig: ViteUserConfig;

  constructor(rawPlugin: Plugin, _farmConfig: UserConfig) {
    this.name = rawPlugin.name;
    this.priority = convertEnforceToPriority(rawPlugin.enforce);
    this._rawPlugin = rawPlugin;
    this._farmConfig = _farmConfig;
    this._viteConfig = farmConfigToViteConfig(_farmConfig);
  }

  // updateModules = {
  //   executor: (result: any) => {
  //     const ctx = {
  //       file: result.paths[0][0]
  //     };
  //     this._rawPlugin.handleHotUpdate(ctx);
  //   }
  // };

  // call both config and configResolved
  config(config: UserConfig) {
    this._farmConfig = config;
    this._viteConfig = farmConfigToViteConfig(config);

    const configHook = this.wrap_raw_plugin_hook(
      'config',
      this._rawPlugin.config
    );

    if (configHook) {
      this._viteConfig = merge(this._viteConfig, configHook(this._viteConfig));
      this._farmConfig = viteConfigToFarmConfig(
        this._viteConfig,
        this._farmConfig
      );
    }

    const configResolvedHook = this.wrap_raw_plugin_hook(
      'configResolved',
      this._rawPlugin.configResolved
    );

    if (configResolvedHook) {
      configResolvedHook(this._viteConfig);
    }

    return this._farmConfig;
  }

  configDevServer(_: DevServer) {
    const hook = this.wrap_raw_plugin_hook(
      'configureServer',
      this._rawPlugin.configureServer
    );

    // TODO: transform farm dev server to vite dev server as much as possible and pass it to the plugin
    if (hook) {
      hook(undefined);
    }
  }

  buildStart = {
    executor: this.wrap_executor(() => {
      const hook = this.wrap_raw_plugin_hook(
        'buildStart',
        this._rawPlugin.buildStart
      );
      hook?.();
    })
  };

  resolve = {
    filters: { sources: ['.*'], importers: ['.*'] },
    executor: this.wrap_executor(
      async (
        params: PluginResolveHookParam,
        context: CompilationContext
      ): Promise<PluginResolveHookResult> => {
        const hook = this.wrap_raw_plugin_hook(
          'resolveId',
          this._rawPlugin.resolveId,
          context
        );
        const absImporterPath = path.resolve(
          process.cwd(),
          params.importer?.relativePath ?? ''
        );
        const resolveIdResult: ResolveIdResult = await hook?.(
          decodeStr(params.source),
          absImporterPath,
          { isEntry: params.kind === 'entry' }
        );
        const removeQuery = (path: string) => {
          const queryIndex = path.indexOf('?');
          if (queryIndex !== -1) {
            return path.slice(0, queryIndex);
          }
          return path.concat('');
        };
        if (isString(resolveIdResult)) {
          return {
            resolvedPath: removeQuery(encodeStr(resolveIdResult)),
            query: customParseQueryString(resolveIdResult),
            sideEffects: false,
            external: false,
            // TODO support meta
            meta: {}
          };
        } else if (isObject(resolveIdResult)) {
          return {
            resolvedPath: removeQuery(encodeStr(resolveIdResult?.id)),
            query: customParseQueryString(resolveIdResult!.id),
            sideEffects: Boolean(resolveIdResult?.moduleSideEffects),
            // TODO support relative and absolute external
            external: Boolean(resolveIdResult?.external),
            meta: {}
          };
        }
        return null;
      }
    )
  };

  load = {
    filters: {
      // TODO support internal filter optimization for common plugins like @vitejs/plugin-vue
      resolvedPaths: ['.*']
    },
    executor: this.wrap_executor(
      async (
        params: PluginLoadHookParam,
        context: CompilationContext
      ): Promise<PluginLoadHookResult> => {
        const hook = this.wrap_raw_plugin_hook(
          'load',
          this._rawPlugin.load,
          context
        );

        const resolvedPath = decodeStr(params.resolvedPath);

        // append query
        const id = formatId(resolvedPath, params.query);
        const result = await hook?.(id);

        if (result) {
          console.log('load', id, result);
          return {
            content: getContentValue(result),
            // only support css as first class citizen for vite plugins
            moduleType: formatLoadModuleType(id)
            // TODO support meta, sourcemap and sideEffects
          };
        }
      }
    )
  };

  transform = {
    filters: {
      resolvedPaths: ['.*']
    },
    executor: this.wrap_executor(
      async (
        params: PluginTransformHookParam,
        context: CompilationContext
      ): Promise<PluginTransformHookResult> => {
        const hook = this.wrap_raw_plugin_hook(
          'transform',
          this._rawPlugin.transform,
          context
        );
        const resolvedPath = decodeStr(params.resolvedPath);
        // append query
        const id = formatId(resolvedPath, params.query);
        const result = await hook?.(params.content, id, {
          // TODO: support ssr
          ssr: false
        });

        if (result) {
          console.log('transform', id, result);

          return {
            content: getContentValue(result),
            sourceMap:
              typeof result.map === 'object'
                ? JSON.stringify(result.map)
                : undefined,
            moduleType: formatTransformModuleType(id)
            // TODO support meta, sourcemap and sideEffects
          };
        }
      }
    )
  };

  private should_execute_plugin() {
    // TODO add command config
    const command =
      this._farmConfig.compilation?.mode === 'production' ? 'build' : 'serve';

    if (typeof this._rawPlugin.apply === 'function') {
      return this._rawPlugin.apply(this._viteConfig, {
        mode: this._farmConfig.compilation.mode,
        command,
        // TODO: ssr build
        ssrBuild: false
      });
    } else if (this._rawPlugin.apply === undefined) {
      return true;
    }

    return this._rawPlugin.apply === command;
  }

  private wrap_executor(executor: (...args: any[]) => any) {
    return async (...args: any[]) => {
      if (this.should_execute_plugin()) {
        return await executor(...args);
      }
    };
  }

  private wrap_raw_plugin_hook(
    hookName: string,
    hook: object | undefined | ((...args: any[]) => any),
    farmContext?: CompilationContext
  ) {
    if (hook === undefined) {
      return undefined;
    }

    if (typeof hook !== 'function') {
      throw new Error(
        `${hookName} hook of vite plugin ${this.name} is not a function. Farm only supports vite plugin with function hooks. This Plugin is not compatible with farm.`
      );
    }

    if (farmContext) {
      const pluginContext = farmContextToViteContext(farmContext);
      return hook.bind(pluginContext);
    } else {
      return hook;
    }
  }
}

// TODO
function farmContextToViteContext(
  _farmContext: CompilationContext
): PluginContext {
  // const viteContext: PluginContext = {
  //   ...farmContext,
  //   getAssetFileName: () => '',
  //   getChunkFileName: () => '',
  //   getFileName: () => '',
  //   getModuleInfo: () => null,
  //   isExternal: () => false,
  //   moduleIds: () => [],
  //   resolve: () => ''
  // };
  // @ts-ignore TODO
  return {};
}

// TODO
function farmConfigToViteConfig(config: UserConfig): ViteUserConfig {
  return {
    root: config.root
  };
}

// TODO
function viteConfigToFarmConfig(
  _config: ViteUserConfig,
  farmConfig: UserConfig
): UserConfig {
  return farmConfig;
}
