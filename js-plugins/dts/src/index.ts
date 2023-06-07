import { JsPlugin, UserConfig } from '@farmfe/core';
import fs from 'node:fs';
import { Project } from 'ts-morph';
import glob from 'fast-glob';

import {
  getResolvedOptions,
  handleExclude,
  handleInclude,
  mergeObjects,
  resolveAbsolutePath,
  getTsConfig
} from './utils.js';

import type { SourceFile, CompilerOptions } from 'ts-morph';
import { relative, resolve } from 'node:path';

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
  const emittedFiles = new Map<string, string>();
  return {
    name: 'farm-dts-plugin',
    config(config: any) {
      farmConfig = config || {};
      root = resolveAbsolutePath(farmConfig.root ?? '', farmConfig.root);
      tsConfigPath = resolveAbsolutePath(tsConfigFilePath, root);
      libFolderPath = libFolderPath && resolveAbsolutePath(libFolderPath, root);
      project = new Project({
        compilerOptions: mergeObjects(compilerOptions, {
          rootDir: compilerOptions.rootDir || root,
          noEmitOnError,
          outDir: farmConfig,
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
        let source = data; // console.log(source);
        return {
          content: source,
          moduleType: 'ts'
        };
      }
    },
    transform: {
      filters: {
        // resolvedPaths: ['.ts$', ...include]
        resolvedPaths: ['.ts$']
      },
      async executor(params: any, ctx: any) {
        // if (project && include && include.length) {
        sourceDtsFiles.add(project.addSourceFileAtPath(params.resolvedPath));
        // project.resolveSourceFileDependencies();
        const dtsOutputFiles = Array.from(sourceDtsFiles).map((sourceFile) => ({
          path: sourceFile.getFilePath(),
          content: sourceFile.getFullText()
        }));

        try {
          const diagnostics = project.getPreEmitDiagnostics();
          console.log(diagnostics);
        } catch (error) {
          console.log(error);
        }
        const service = project.getLanguageService();
        // const outputFiles = project
        //   .getSourceFiles()
        //   .map((sourceFile) =>
        //     service
        //       .getEmitOutput(sourceFile, true)
        //       .getOutputFiles()
        //       .map((outputFile) => ({
        //         path: resolve(
        //           root,
        //           relative(
        //             farmConfig.output.path,
        //             outputFile.compilerObject.name
        //           )
        //         ),
        //         content: outputFile.getText()
        //       }))
        //   )
        //   .flat()
        //   .concat(dtsOutputFiles);
        // console.log(outputFiles);

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
        let source = '';
        return {
          content: source,
          moduleType: 'ts'
        };
      }
    }
  };
}
