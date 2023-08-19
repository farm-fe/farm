import { JsPlugin, UserConfig } from '@farmfe/core';
import fs from 'fs-extra';
import { dirname } from 'node:path';
import os from 'node:os';
import { Project } from 'ts-morph';
import glob from 'fast-glob';

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

import type { SourceFile, CompilerOptions } from 'ts-morph';
import path, { relative, resolve } from 'node:path';

const noneExport = 'export {};\n';
const tsRE = /\.(m|c)?tsx?$/;
const jsRE = /\.(m|c)?jsx?$/;
const dtsRE = /\.d\.(m|c)?tsx?$/;
const tjsRE = /\.(m|c)?(t|j)sx?$/;
export default function farmDtsPlugin(options: any = {}): JsPlugin {
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
  let allowJs = false;
  // let exclude = handleExclude(options);
  // let include: any = handleInclude(options);
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
      isDev = config.mode === 'development';
      farmConfig = config;
      root = config.root || process.cwd();
      outputDir = ensureAbsolute(
        options.outputDir ? options.outputDir : config.output.path,
        root
      );
      tsConfigPath = resolveAbsolutePath(tsconfigPath, root);

      libFolderPath = libFolderPath && ensureAbsolute(libFolderPath, root);

      const mergeCompilerOptions = {
        compilerOptions: mergeObjects(compilerOptions, {
          rootDir: compilerOptions.rootDir || root,
          noEmitOnError: false,
          outDir: outputDir,
          declarationDir: undefined,
          noUnusedParameters: false,
          declaration: true,
          noEmit: false,
          emitDeclarationOnly: true,
          composite: false
        } as CompilerOptions),
        tsConfigFilePath: tsConfigPath,
        skipAddingFilesFromTsConfig: true,
        libFolderPath
      };
      project = new Project(mergeCompilerOptions);
      tsConfigOptions = getTsConfig(
        tsConfigPath,
        project.getFileSystem().readFileSync
      );
      include = ensureArray(
        options.include ?? tsConfigOptions.include ?? '**/*'
      ).map(normalizeGlob);
      exclude = ensureArray(
        options.exclude ?? tsConfigOptions.exclude ?? 'node_modules/**'
      ).map(normalizeGlob);
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
        project.createSourceFile(
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
        if (project && include && include.length) {
          const files = await glob(include, {
            cwd: root,
            absolute: true,
            ignore: exclude
          });

          for (const file of files) {
            if (dtsRE.test(file)) {
              sourceDtsFiles.add(project.addSourceFileAtPath(file));
              if (!copyDtsFiles) {
                continue;
              }

              // includedFiles.add(file);
              continue;
            }

            // includedFiles.add(
            //   `${file.replace(tjsRE, '')}.d.${extPrefix(file)}ts`
            // );
          }

          project.compilerOptions.set({ allowJs: true });
        }

        const dtsOutputFiles = Array.from(sourceDtsFiles).map((sourceFile) => ({
          path: sourceFile.getFilePath(),
          content: sourceFile.getFullText()
        }));
        const startTime = Date.now();
        if (!skipDiagnostics) {
          const diagnostics = project.getPreEmitDiagnostics();
          if (diagnostics?.length) {
            console.warn(
              project.formatDiagnosticsWithColorAndContext(diagnostics)
            );
          }
          if (typeof afterDiagnostic === 'function') {
            const result = afterDiagnostic(diagnostics);
            isPromise(result) && (await result);
          }
        }
        project.resolveSourceFileDependencies();

        const service = project.getLanguageService();
        const outputFiles = project
          .getSourceFiles()
          .map((sourceFile) =>
            service
              .getEmitOutput(sourceFile, true)
              .getOutputFiles()
              .map((outputFile) => ({
                path: resolve(
                  root,
                  relative(outputDir, outputFile.compilerObject.name)
                ),
                content: outputFile.getText()
              }))
          )
          .flat()
          .concat(dtsOutputFiles);

        entryRoot =
          entryRoot ||
          queryPublicPath(outputFiles.map((file: any) => file.path));
        entryRoot = ensureAbsolute(entryRoot, root);
        await runParallel(
          os.cpus().length,
          outputFiles,
          async (outputFile: any) => {
            let filePath = outputFile.path;
            filePath = resolve(outputDir, relative(entryRoot, filePath));
            let content = outputFile.content;
            writeFileWithCheck(filePath, content);
          }
        );
        console.warn(
          `${'[Farm:dts]'} Declaration files built in ${
            Date.now() - startTime
          }ms.`
        );
        return {};
      }
    }
  };
}

async function writeFileWithCheck(filePath: string, content: string) {
  // 获取文件夹路径
  const folderPath = path.dirname(filePath);
  try {
    // 检查文件夹是否存在
    await fs.access(folderPath);
  } catch (error) {
    // 如果文件夹不存在，则创建它
    await fs.mkdir(folderPath, { recursive: true });
  }

  // 写文件
  await fs.writeFile(filePath, content, 'utf-8');
}
