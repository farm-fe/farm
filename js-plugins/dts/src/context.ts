import type { UserConfig } from '@farmfe/core';
import { DefaultLogger } from './logger.js';
import chalk from 'chalk';
import glob from 'fast-glob';
import os from 'node:os';
import { relative, resolve } from 'node:path';
import { CompilerOptions, Project, SourceFile } from 'ts-morph';
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
  tryToReadFileSync,
  writeFileWithCheck
} from './utils.js';
export default class Context {
  config: UserConfig['compilation'];
  options: any;
  project: Project | undefined;
  include: string[];
  exclude: string[];
  logger = new DefaultLogger({ name: 'FarmDtsPlugin' });
  handleResolveOptions(options: any = {}, config: UserConfig) {
    this.config = config;
    let libFolderPath: string;
    const defaultOption: any = {
      tsconfigPath: 'tsconfig.json',
      aliasesExclude: [],
      staticImport: false,
      clearPureImport: true,
      insertTypesEntry: false,
      noEmitOnError: false,
      skipDiagnostics: true,
      copyDtsFiles: false,
      afterDiagnostic: () => {}
    };

    const userOptions = mergeObjects(defaultOption, options);
    const isDev = this.config.mode === 'development';
    const root = this.config.root || process.cwd();
    const sourceDtsFiles: any = new Set<SourceFile>();
    const outputFiles = new Map<string, string>();
    const emittedFiles = new Map<string, string>();
    const outputDir = ensureAbsolute(
      options.outputDir ? options.outputDir : this.config.output.path,
      root
    );
    const aliasesExclude = userOptions?.aliasesExclude ?? [];
    const tsConfigPath = resolveAbsolutePath(userOptions.tsconfigPath, root);
    libFolderPath = libFolderPath && ensureAbsolute(libFolderPath, root);
    const compilerOptions = userOptions.compilerOptions ?? {};

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
    this.project = new Project(mergeCompilerOptions);
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

    const aliasOptions: any = config?.compilation?.resolve?.alias ?? [];
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
    console.log(aliases);

    this.options = {
      ...userOptions,
      isDev,
      root,
      outputDir,
      sourceDtsFiles,
      outputFiles,
      emittedFiles,
      tsConfigOptions,
      tsConfigPath
    };
  }

  async handleTransform(id: string) {
    this.project.createSourceFile(id, await tryToReadFileSync(id), {
      overwrite: true
    });
  }

  async handleCloseBundle() {
    if (this.project && this.include && this.include.length) {
      const files = await glob(this.include, {
        cwd: this.options.root,
        absolute: true,
        ignore: this.exclude
      });
      const dtsRE = /\.d\.(m|c)?tsx?$/;
      for (const file of files) {
        if (dtsRE.test(file)) {
          this.options.sourceDtsFiles.add(
            this.project.addSourceFileAtPath(file)
          );
          if (!this.options.copyDtsFiles) {
            continue;
          }

          // includedFiles.add(file);
          continue;
        }

        // includedFiles.add(
        //   `${file.replace(tjsRE, '')}.d.${extPrefix(file)}ts`
        // );
      }

      this.project.compilerOptions.set({ allowJs: true });
    }

    const dtsOutputFiles = Array.from(this.options.sourceDtsFiles).map(
      (sourceFile: any) => ({
        path: sourceFile.getFilePath(),
        content: sourceFile.getFullText()
      })
    );
    const startTime = performance.now();
    if (!this.options.skipDiagnostics) {
      const diagnostics = this.project.getPreEmitDiagnostics();
      if (diagnostics?.length) {
        console.warn(
          this.project.formatDiagnosticsWithColorAndContext(diagnostics)
        );
      }
      if (typeof this.options.afterDiagnostic === 'function') {
        const result = this.options.afterDiagnostic(diagnostics);
        isPromise(result) && (await result);
      }
    }
    this.project.resolveSourceFileDependencies();

    const service = this.project.getLanguageService();
    const outputFiles = this.project
      .getSourceFiles()
      .map((sourceFile) =>
        service
          .getEmitOutput(sourceFile, true)
          .getOutputFiles()
          .map((outputFile) => ({
            path: resolve(
              this.options.root,
              relative(this.options.outputDir, outputFile.compilerObject.name)
            ),
            content: outputFile.getText()
          }))
      )
      .flat()
      .concat(dtsOutputFiles);
    let entryRoot = this.options.entryRoot ?? '';
    entryRoot =
      entryRoot || queryPublicPath(outputFiles.map((file: any) => file.path));
    entryRoot = ensureAbsolute(entryRoot, this.options.root);
    await runParallel(os.cpus().length, outputFiles, async (outputFile) => {
      let filePath = outputFile.path;
      filePath = resolve(this.options.outputDir, relative(entryRoot, filePath));
      let content = outputFile.content;
      content = transformAliasImport(
        filePath,
        content,
        this.options.aliases,
        this.options.aliasesExclude
      );

      writeFileWithCheck(filePath, content);
    });
    const endTime = performance.now();
    const elapsedTime = Math.floor(endTime - startTime);

    this.logger.info(
      `⚡️ Dts Plugin Build completed in ${chalk.bold(
        chalk.green(`${elapsedTime}ms`)
      )}! Resources emitted to ${chalk.green(this.config.output.path)}.`
    );
  }
}
