import os from 'node:os';
import { relative, resolve } from 'node:path';
import type { UserConfig } from '@farmfe/core';
import chalk from 'chalk';
import glob from 'fast-glob';
import { CompilerOptions, Project, SourceFile } from 'ts-morph';
import { DefaultLogger } from './logger.js';
import {
  ensureAbsolute,
  ensureArray,
  getTsConfig,
  isObject,
  isPromise,
  isRegExp,
  mergeObjects,
  normalizeGlob,
  queryPublicPath,
  resolveAbsolutePath,
  runParallel,
  transformAliasImport,
  writeFileWithCheck
} from './utils.js';
export default class Context {
  config: UserConfig['compilation'] & { root?: string };
  options: any;
  project: Project | undefined;
  include: string[];
  exclude: string[];
  logger = new DefaultLogger({ name: 'FarmDtsPlugin' });
  handleResolveOptions(options: any = {}, config: UserConfig['compilation']) {
    this.config = config;
    let libFolderPath: string;
    const defaultOption: any = {
      tsconfigPath: 'tsconfig.json',
      aliasesExclude: [],
      staticImport: false,
      clearPureImport: true,
      insertTypesEntry: false,
      noEmitOnError: false,
      skipDiagnostics: false,
      copyDtsFiles: false,
      afterDiagnostic: () => ({})
    };

    const userOptions = mergeObjects(defaultOption, options);
    const isDev = this.config.mode === 'development';
    const root = this.config.root || process.cwd();
    const sourceDtsFiles: Set<SourceFile> = new Set<SourceFile>();
    const outputFiles = new Map<string, string>();
    const emittedFiles = new Map<string, string>();
    const outputDir = ensureAbsolute(
      options.outputDir ? options.outputDir : this.config.output?.path,
      root
    );
    const outDir = options.outputDir
      ? options.outputDir
      : this.config.output?.path;
    const aliasesExclude = userOptions?.aliasesExclude ?? [];
    const tsConfigPath = resolveAbsolutePath(userOptions.tsconfigPath, root);
    const folderPath = libFolderPath && ensureAbsolute(libFolderPath, root);
    const compilerOptions = userOptions.compilerOptions ?? {};

    const tsCompilerOptions = {
      compilerOptions: mergeObjects(compilerOptions, {
        rootDir: compilerOptions.rootDir || root,
        noEmitOnError: false,
        outDir: outputDir,
        declarationDir: undefined,
        noUnusedParameters: false,
        declaration: true,
        noEmit: false,
        emitDeclarationOnly: true,
        composite: false,
        allowJs: true
      } as CompilerOptions),
      tsConfigFilePath: tsConfigPath,
      skipAddingFilesFromTsConfig: true,
      libFolderPath: folderPath
    };

    this.project = new Project(tsCompilerOptions);
    const tsConfigOptions = getTsConfig(
      tsConfigPath,
      this.project.getFileSystem().readFileSync
    );
    this.include = ensureArray(
      options.include ?? tsConfigOptions.include ?? '**/*'
    ).map(normalizeGlob);
    this.exclude = ensureArray(
      options.exclude ?? tsConfigOptions.exclude ?? 'node_modules/**'
    ).map(normalizeGlob);

    const aliasOptions: UserConfig['compilation']['resolve']['alias'] =
      config?.resolve?.alias ?? {};
    let aliases: any[] = [];
    if (isObject(aliasOptions)) {
      aliases = Object.entries(aliasOptions).map(([key, value]) => {
        return { find: key, replacement: value };
      });
    } else {
      aliases = ensureArray(aliasOptions);
    }

    if (aliasesExclude.length > 0) {
      aliases = aliases.filter(
        ({ find }) =>
          !aliasesExclude.some(
            (alias: any) =>
              alias &&
              (isRegExp(find)
                ? find.toString() === alias.toString()
                : isRegExp(alias)
                  ? find.match(alias)?.[0]
                  : find === alias)
          )
      );
    }

    this.options = {
      ...userOptions,
      isDev,
      root,
      outputDir: outDir,
      outputDirPath: outputDir,
      sourceDtsFiles,
      outputFiles,
      emittedFiles,
      tsConfigOptions,
      tsConfigPath,
      aliases,
      aliasOptions,
      aliasesExclude
    };
  }

  async handleTransform(id: string, content: string) {
    this.project.createSourceFile(id, content, {
      overwrite: true
    });
  }

  async handleCloseBundle() {
    // handle already dts files in file system
    const dtsOutputFiles = await this.handleAlreadyExistDtsFile();

    this.handleDoctor();
    // use doctor to check if there are any diagnostics
    const startTime = performance.now();

    // get all source files and resolve current file dependencies
    this.project.resolveSourceFileDependencies();

    const service = this.project.getLanguageService();

    const outputFiles = this.project
      .getSourceFiles()
      .flatMap((sourceFile) =>
        service
          .getEmitOutput(sourceFile, true)
          .getOutputFiles()
          .filter((outputFile) =>
            outputFile.compilerObject.name.startsWith(this.options.root)
          )
          .map((outputFile) => ({
            path: resolve(
              this.options.root,
              relative(this.options.outputDir, outputFile.compilerObject.name)
            ),
            content: outputFile.getText()
          }))
      )
      .concat(dtsOutputFiles);

    let entryRoot = this.options.entryRoot ?? '';
    entryRoot =
      entryRoot ||
      queryPublicPath(
        outputFiles.map((file: { path: string; content: string }) => file.path)
      );
    entryRoot = ensureAbsolute(entryRoot, this.options.root);

    await runParallel(os.cpus().length, outputFiles, async (outputFile) => {
      let filePath = outputFile.path;

      let content = outputFile.content;

      content = transformAliasImport(
        filePath,
        content,
        this.options.aliases,
        this.options.aliasesExclude
      );
      filePath = resolve(this.options.outputDir, relative(entryRoot, filePath));

      writeFileWithCheck(filePath, content);
    });

    const elapsedTime = Math.floor(performance.now() - startTime);

    this.logger.info(
      `⚡️ Dts Plugin Build completed in ${chalk.bold(
        chalk.green(`${elapsedTime}ms`)
      )}! Resources emitted to ${chalk.bold(
        chalk.green(this.options.outputDir)
      )}.`
    );
  }

  async handleAlreadyExistDtsFile() {
    if (this.project && this.include && this.include.length) {
      const files = await glob(this.include, {
        cwd: this.options.root,
        absolute: true,
        ignore: this.exclude
      });

      // check if there are dts files in the project
      const dtsRE = /\.d\.(m|c)?tsx?$/;
      for (const file of files) {
        if (dtsRE.test(file)) {
          this.options.sourceDtsFiles.add(
            this.project.addSourceFileAtPath(file)
          );
        }
      }
    }

    const dtsOutputFiles = Array.from(this.options.sourceDtsFiles).map(
      (sourceFile: any) => ({
        path: sourceFile.getFilePath(),
        content: sourceFile.getFullText()
      })
    );

    return dtsOutputFiles;
  }

  async handleDoctor() {
    if (!this.options.skipDiagnostics) {
      const diagnostics = this.project.getPreEmitDiagnostics();
      if (diagnostics?.length) {
        this.logger.warn(
          this.project.formatDiagnosticsWithColorAndContext(diagnostics)
        );
      }
      if (typeof this.options.afterDiagnostic === 'function') {
        const result = this.options.afterDiagnostic(diagnostics);
        isPromise(result) && (await result);
      }
    }
  }
}
