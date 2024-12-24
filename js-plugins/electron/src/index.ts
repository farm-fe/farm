import fs from 'node:fs';
import type { AddressInfo } from 'node:net';
import path from 'node:path';
// TODO: submit a PR to farm(export default farm)
import {
  type FarmCLIOptions,
  type JsPlugin,
  type Server,
  type UserConfig,
  build as farmBuild
} from '@farmfe/core';

export interface BuildOptions {
  /**
   * Shortcut of `compilation.input`
   */
  input: string | Record<string, string>;
  farm?: UserConfig;
}

export interface ElectronPluginOptions {
  main: BuildOptions;
  preload?: BuildOptions;
}

type ElectronOptionsKey = keyof ElectronPluginOptions;

export default function farmElectronPlugin(
  options: ElectronPluginOptions
): JsPlugin {
  const builds = {
    main: options.main,
    preload: options.preload
  };
  let isDev: boolean;

  return {
    name: 'farm-plugin-electron',
    config(config) {
      config.compilation ??= {};
      config.compilation.output ??= {};
      config.compilation.output.publicPath ??= './';

      // Not work
      // config.compilation.assets ??= {}
      // config.compilation.assets.publicDir ??= ''
      return config;
    },
    configureDevServer(server) {
      isDev = true;

      server.server?.once('listening', () => {
        // Used in electron/main.ts for during dev
        process.env.FARM_DEV_SERVER_URL = resolveServerUrl(server);

        for (const [name, opts] of Object.entries(builds)) {
          if (!opts) continue;

          opts.farm ??= {};
          opts.farm.plugins ??= [];
          opts.farm.plugins.push({
            name: ':startup',
            finish: {
              async executor() {
                if (name === 'preload') {
                  // Hot reload preload scripts
                  server.ws.send({ type: 'full-reload' });
                } else {
                  // Hot restart main process
                  Starter.start();
                }
              }
            }
          });
          /* await */ farmBuild(
            resolveFarmConfig(name as ElectronOptionsKey, opts, server)
          );
        }
      });
    },
    finish: {
      async executor() {
        if (isDev) return;

        for (const [name, opts] of Object.entries(builds)) {
          if (!opts) continue;
          if (name === 'preload') {
            // TODO: .mjs file ext
          }
          farmBuild(resolveFarmConfig(name as ElectronOptionsKey, opts));
        }
      }
    }
  };
}

function resolveFarmConfig(
  name: ElectronOptionsKey,
  opts: BuildOptions,
  server?: Server
) {
  const input =
    typeof opts.input === 'string'
      ? { [path.parse(opts.input).name]: opts.input }
      : opts.input;
  const isEsm = resolvePackageJson()?.type === 'module';
  const isDev = !!server;

  opts.farm ??= {};
  opts.farm.compilation ??= {};
  opts.farm.compilation.minify ??= !isDev;
  opts.farm.compilation.input = input;
  opts.farm.compilation.output ??= {};
  opts.farm.compilation.output.path ??= 'dist-electron';
  opts.farm.compilation.output.targetEnv ??= 'node16';
  opts.farm.compilation.external ??= [];
  opts.farm.compilation.external.push('^electron$');
  opts.farm.compilation.watch ??= isDev;
  opts.farm.compilation.partialBundling ??= {};
  opts.farm.compilation.partialBundling.enforceResources ??= [];
  opts.farm.plugins ??= [];

  if (!opts.farm.compilation.output.format) {
    opts.farm.compilation.output.format =
      name === 'preload'
        ? // In most cases, preload scripts use `cjs` format
          'cjs'
        : isEsm
          ? 'esm'
          : 'cjs';
  }

  if (!opts.farm.compilation.output.entryFilename) {
    opts.farm.compilation.output.entryFilename =
      name === 'preload'
        ? // https://www.electronjs.org/docs/latest/tutorial/esm#esm-preload-scripts-must-have-the-mjs-extension
          isEsm
          ? '[entryName].mjs'
          : '[entryName].js'
        : '[entryName].js';
  }

  if (name === 'preload') {
    opts.farm.compilation.partialBundling.enforceResources.push({
      test: ['.+'],
      name: 'preload'
    });

    if (isEsm) {
      opts.farm.plugins.push({
        name: 'farm-plugin-electron:preload-scripts-runtime',
        renderResourcePot: {
          filters: {
            resourcePotTypes: ['js'],
            moduleIds: []
          },
          async executor(param) {
            return {
              ...param,
              // Fix runtime code error `__filename is not defined`
              // TODO: `import.meta.url` of `__filename` will be converted to filename(absolute path) when targetEnv=node`.
              content: `var electron_preload_scripts_filename='';globalThis['__' + 'filename']=electron_preload_scripts_filename;${param.content}`
            };
          }
        }
      });
    }
  }

  // TODO: submit a PR to farm(Omit<FarmCLIOptions, 'server'> & UserConfig)
  return opts.farm as FarmCLIOptions;
}

function resolveServerUrl(server: Server) {
  const addressInfo = server.server?.address();
  const isAddressInfo = (x: any): x is AddressInfo => x?.address;

  if (isAddressInfo(addressInfo)) {
    const { port } = addressInfo;

    return `http://localhost:${port}`;
  }
}

class Starter {
  static electronApp: import('node:child_process').ChildProcess | undefined;
  static hookedProcessExit = false;

  static exit = async () => {
    if (this.electronApp) {
      this.electronApp.removeAllListeners();
      if (this.electronApp.pid) {
        process.kill(this.electronApp.pid);
      }
    }
  };

  static start = async () => {
    const { spawn } = await import('node:child_process');
    const electron = await import('electron');
    const electronPath = <any>(electron.default ?? electron);

    await this.exit();

    // Start Electron.app
    this.electronApp = spawn(electronPath, ['.', '--no-sandbox'], {
      stdio: 'inherit'
    });

    // Exit command after Electron.app exits
    this.electronApp.once('exit', process.exit);

    if (!this.hookedProcessExit) {
      this.hookedProcessExit = true;
      process.once('exit', this.exit);
    }
  };
}

function resolvePackageJson(root = process.cwd()): {
  type?: 'module' | 'commonjs';
  [key: string]: any;
} | null {
  const packageJsonPath = path.join(root, 'package.json');
  const packageJsonStr = fs.readFileSync(packageJsonPath, 'utf8');
  try {
    return JSON.parse(packageJsonStr);
  } catch {
    return null;
  }
}
