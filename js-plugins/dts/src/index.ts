import { JsPlugin, UserConfig } from '@farmfe/core';
import fs from 'fs-extra';
import { Project } from 'ts-morph';
import glob from 'fast-glob';
import os from 'node:os';
import { dirname } from 'node:path';

import {
  handleExclude,
  handleInclude,
  mergeObjects,
  resolveAbsolutePath,
  getTsConfig,
  ensureArray,
  ensureAbsolute,
  queryPublicPath,
  runParallel
} from './utils.js';
import path, { relative, resolve } from 'node:path';
import Context from './context.js';
import type { SourceFile, CompilerOptions } from 'ts-morph';

const noneExport = 'export {};\n';
const tsRE = /\.(m|c)?tsx?$/;
const jsRE = /\.(m|c)?jsx?$/;
const dtsRE = /\.d\.(m|c)?tsx?$/;
const tjsRE = /\.(m|c)?(t|j)sx?$/;
export default function farmDtsPlugin(options: any = {}): JsPlugin {
  const ctx = new Context();
  // options hooks to get farmConfig
  let farmConfig: UserConfig['compilation'];
  const {
    tsConfigFilePath = 'tsconfig.json',
    tsconfigPath = 'tsconfig.json',
    aliasesExclude = [],
    staticImport = false,
    clearPureImport = true,
    insertTypesEntry = false,
    noEmitOnError = false,
    skipDiagnostics = true,
    copyDtsFiles = false,
    afterDiagnostic = () => {}
  } = options;

  let isDev: boolean = false;
  let root = ensureAbsolute(options.root ?? '', process.cwd());
  let publicRoot = '';
  let entryRoot = options.entryRoot ?? '';
  let libFolderPath: string;
  let tsConfigOptions;
  let compilerOptions = options.compilerOptions ?? {};
  let project: Project | undefined;
  let outputDir: string;
  let tsConfigPath: string;
  let include: string[];
  let exclude: string[];
  const sourceDtsFiles = new Set<SourceFile>();
  const outputFiles = new Map<string, string>();
  const emittedFiles = new Map<string, string>();

  async function tryToReadFileSync(path: string) {
    try {
      return await fs.promises.readFile(path, 'utf-8');
    } catch (error) {
      console.error(`[Farm Plugin Solid]: ${error.type}: ${error.message}`);
    }
  }
  const globSuffixRE = /^((?:.*\.[^.]+)|(?:\*+))$/;

  function normalizeGlob(path: string) {
    if (/[\\/]$/.test(path)) {
      return path + '**';
    } else if (!globSuffixRE.test(path.split(/[\\/]/).pop()!)) {
      return path + '/**';
    }

    return path;
  }

  function isPromise(value: unknown): value is Promise<any> {
    return (
      !!value &&
      typeof (value as any).then === 'function' &&
      typeof (value as any).catch === 'function'
    );
  }

  // TODO support vue
  return {
    name: 'farm-dts-plugin',
    priority: 1000,
    config(config: any) {
      ctx.handleResolveOptions(options, config);
      return config;
    },
    load: {
      filters: {
        resolvedPaths: ['.ts$']
      },
      async executor(params) {
        const { resolvedPath } = params;
        const content = await tryToReadFileSync(resolvedPath);
        return {
          content,
          moduleType: 'dts'
        };
      }
    },
    transform: {
      filters: {
        moduleTypes: ['dts']
      },
      async executor(params) {
        const { resolvedPath, content } = params;
        ctx.project.createSourceFile(
          resolvedPath,
          await tryToReadFileSync(resolvedPath),
          { overwrite: true }
        );
        return {
          content,
          moduleType: 'ts'
        };
      }
    },
    finish: {
      async executor() {
        ctx.handleCloseBundle();
        return {};
      }
    }
  };
}
