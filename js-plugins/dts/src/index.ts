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
    skipDiagnostics = false,
    copyDtsFiles = false
  } = options;
  let useFlag: boolean = false;
  let root = ensureAbsolute(options.root ?? '', process.cwd());
  let publicRoot = '';
  let entryRoot = options.entryRoot ?? '';
  let libFolderPath: string;
  let compilerOptions = options.compilerOptions ?? {};
  let project: Project | undefined;
  let outputDirs: string[];
  let tsConfigPath: string;
  let allowJs = false;
  // let exclude = handleExclude(options);
  // let include: any = handleInclude(options);
  const sourceDtsFiles = new Set<SourceFile>();
  const outputFiles = new Map<string, string>();
  const emittedFiles = new Map<string, string>();

  // TODO support vue
  return {
    name: 'farm-dts-plugin',
    priority: 1000,
    config(config: any) {
      useFlag = config.mode !== 'development';
      farmConfig = config || {};

      root = config.root || process.cwd();

      outputDirs = options.outputDir
        ? ensureArray(options.outputDir).map((d) => ensureAbsolute(d, root))
        : [ensureAbsolute(config.output.path, root)];

      tsConfigPath = resolveAbsolutePath(tsconfigPath, root);
      // libFolderPath = resolveAbsolutePath(libFolderPath, root);
      libFolderPath = undefined;
      const oo = {
        compilerOptions: mergeObjects(compilerOptions, {
          rootDir: compilerOptions.rootDir || root,
          noEmitOnError: false,
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
      };
      project = new Project(oo);
      console.log(oo);

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
        // console.log('load', params.resolvedPath);
        const data = await fs.promises.readFile(resolvedPath, 'utf-8');
        if (project) {
          // project.createSourceFile(
          //   params.resolvedPath,
          //   await fs.readFile(params.resolvedPath, 'utf-8'),
          //   { overwrite: true }
          // );
          // TODO never use 好像就不需要了
          // const files = await glob(['src/**', '*.d.ts'], {
          //   cwd: root,
          //   absolute: true,
          //   ignore: ['node_modules/**']
          // });
          // for (const file of files) {
          //   // if (dtsRE.test(file)) {
          //   sourceDtsFiles.add(project.addSourceFileAtPath(file));
          //   if (!copyDtsFiles) {
          //     continue;
          //   }
          //   // includedFiles.add(file);
          //   continue;
          //   // }
          // }
        }
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
        console.log(params.resolvedPath);
        project.createSourceFile(
          params.resolvedPath,
          await fs.readFile(params.resolvedPath, 'utf-8'),
          { overwrite: true }
        );
        return {
          content: params.content,
          moduleType: 'ts'
        };
      }
    },
    finish: {
      async executor() {
        console.log('我要开始构建了啊');
        // console.log(sourceDtsFiles.size);

        // const dtsOutputFiles = Array.from(sourceDtsFiles).map((sourceFile) => ({
        //   path: sourceFile.getFilePath(),
        //   content: sourceFile.getFullText()
        // }));

        try {
          const diagnostics = project.getPreEmitDiagnostics();
        } catch (error) {
          console.log(error);
        }
        const startTime = Date.now();
        // project.resolveSourceFileDependencies();
        // console.log(project.getSourceFiles());
        const outputDir = outputDirs[0];

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
          .flat();
        // .concat(dtsOutputFiles);
        // console.log('outputFiles', outputFiles);

        entryRoot =
          entryRoot ||
          queryPublicPath(outputFiles.map((file: any) => file.path));
        entryRoot = ensureAbsolute(entryRoot, root);
        await runParallel(
          os.cpus().length,
          outputFiles,
          async (outputFile: any) => {
            let filePath = outputFile.path;

            filePath = resolve(outputDirs[0], relative(entryRoot, filePath));
            let content = outputFile.content;

            if (filePath.endsWith('.d.ts')) {
              writeFileWithCheck(filePath, content);
            }
          }
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
