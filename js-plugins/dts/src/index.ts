import { JsPlugin, UserConfig } from '@farmfe/core';
import fs from 'fs-extra';
import { dirname } from 'node:path';
import os from 'node:os';
import { Project } from 'ts-morph';
import glob from 'fast-glob';

import {
  getResolvedOptions,
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

export default function farmDtsPlugin(
  farmDtsPluginOptions: any = {}
): JsPlugin {
  // options hooks to get farmConfig
  let farmConfig: UserConfig['compilation'];
  let tsConfigPath: string;
  let root: string;
  let libFolderPath: string;
  let allowJs: boolean;
  const resolvedOptions = getResolvedOptions(farmDtsPluginOptions);
  const {
    tsConfigFilePath = 'tsconfig.json',
    noEmitOnError = false,
    skipDiagnostics = false,
    copyDtsFiles = false
  } = resolvedOptions;
  let compilerOptions = resolvedOptions.compilerOptions ?? {};
  let project: Project | undefined;
  let exclude = handleExclude(resolvedOptions);
  let include = handleInclude(resolvedOptions);
  const sourceDtsFiles = new Set<SourceFile>();
  let entryRoot = farmDtsPluginOptions.entryRoot ?? '';
  let outputDirs: string[] = [];
  const emittedFiles = new Map<string, string>();

  // CONST
  const noneExport = 'export {};\n';

  // TODO support vue
  return {
    name: 'farm-dts-plugin',
    priority: 1000,
    config(config: any) {
      farmConfig = config || {};
      root = config.root || process.cwd();

      outputDirs = farmDtsPluginOptions.outputDir
        ? ensureArray(farmDtsPluginOptions.outputDir).map((d) =>
            ensureAbsolute(d, root)
          )
        : [ensureAbsolute(config.output.path, root)];

      tsConfigPath = resolveAbsolutePath(tsConfigFilePath, root);
      libFolderPath = resolveAbsolutePath(libFolderPath, root);
      project = new Project({
        compilerOptions: mergeObjects(compilerOptions, {
          rootDir: compilerOptions.rootDir || root,
          noEmitOnError,
          outDir: outputDirs[0],
          // #27 declarationDir option will make no declaration file generated
          declarationDir: undefined,
          // compile vue setup script will generate expose parameter for setup function
          // although user never use it which will get an unexpected unused error
          noUnusedParameters: false,
          declaration: true,
          noEmit: false,
          emitDeclarationOnly: true,
          // #153 maybe a bug of ts-morph
          composite: false
        } as CompilerOptions),
        tsConfigFilePath: tsConfigPath,
        skipAddingFilesFromTsConfig: true,
        libFolderPath
      });
      allowJs = project.getCompilerOptions().allowJs ?? false;
      const tsConfig = getTsConfig(
        tsConfigPath,
        project.getFileSystem().readFileSync
      );
      compilerOptions = tsConfig.compilerOptions;
      return config;
    },
    load: {
      filters: {
        resolvedPaths: ['.ts$']
      },
      async executor(params: any, ctx: any) {
        const { resolvedPath } = params;
        const data = await fs.promises.readFile(resolvedPath, 'utf-8');
        return {
          content: data,
          moduleType: 'dts'
        };
      }
    },
    transform: {
      filters: {
        // resolvedPaths: ['.ts$'],
        moduleTypes: ['dts']
      },
      async executor(params: any, ctx: any) {
        if (project) {
          sourceDtsFiles.add(project.addSourceFileAtPath(params.resolvedPath));
          // project.resolveSourceFileDependencies();
          const dtsOutputFiles = Array.from(sourceDtsFiles).map(
            (sourceFile) => ({
              path: sourceFile.getFilePath(),
              content: sourceFile.getFullText()
            })
          );

          try {
            const diagnostics = project.getPreEmitDiagnostics();
            // console.log(diagnostics);
          } catch (error) {
            console.log(error);
          }
          const service = project.getLanguageService();
          const outputFiles = project
            .getSourceFiles()
            .map((sourceFile) =>
              service
                .getEmitOutput(sourceFile, true)
                .getOutputFiles()
                .map((outputFile) => {
                  return {
                    path: resolve(
                      root,
                      relative(
                        farmConfig.output.path,
                        path.normalize(outputFile.compilerObject.name)
                      )
                    ),
                    content: outputFile.getText()
                  };
                })
            )
            .flat();
          // .concat(dtsOutputFiles);
          console.log(outputFiles);

          entryRoot =
            entryRoot || queryPublicPath(outputFiles.map((file) => file.path));
          entryRoot = ensureAbsolute(entryRoot, root);
          await runParallel(
            os.cpus().length,
            outputFiles,
            async (outputFile: any) => {
              let filePath = outputFile.path;
              filePath = resolve(outputDirs[0], relative(entryRoot, filePath));
              let content = outputFile.content;
              if (filePath.endsWith('.d.ts')) {
                console.log('找到了');

                writeFileWithCheck(filePath, content);
              }
            }
          );
          // }
          // console.log(params);
          // const project = new Project();
          // console.log(project);
          // const sourceFile = project.addSourceFileAtPath(params.resolvedPath);
          // // const result = project.emitToMemory();
          // const result = await project.emit({ emitOnlyDtsFiles: true });
          // // const dtsFile
          // const project = new Project({
          //   compilerOptions: { outDir: 'dist', declaration: true }
          // });
          // project.createSourceFile('MyFile.ts', params.content);
          // project.createSourceFile(params.resolvedPath, params.content);
          // project.emit(); // async
          // const dtsFile = sourceFile
          //   .emitToMemory()
          //   .getFiles()
          //   .find((f) => f.filePath.endsWith('.d.ts'))!;
          // console.log(dtsFile.text);
        }
        return {
          content: params.content,
          moduleType: 'ts'
        };
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
