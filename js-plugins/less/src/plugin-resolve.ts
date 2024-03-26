import type { CompilationContext } from '@farmfe/core';
import type Less from 'less';
import path from 'node:path';

let CustomLessManager: any;

/// This Less plugin is Vite compatible. The behavior is aligned with the internal Less handler in Vite.
export function createLessResolvePlugin(
  less: typeof Less,
  ctx: CompilationContext,
  resolvedPath: string
): Less.Plugin {
  const { FileManager } = less;

  CustomLessManager ??= class LessManager extends FileManager {
    rootPath: string;
    constructor(rootPath: string) {
      super();
      this.rootPath = rootPath;
    }
    override supports(filename: string) {
      return !/^(?:https?:)?\/\//.test(filename);
    }
    override supportsSync() {
      return false;
    }

    override async loadFile(
      filename: string,
      dir: string,
      opts: any,
      env: any
    ): Promise<Less.FileLoadResult> {
      const result = await ctx.resolve(
        {
          source: filename,
          importer: path.join(dir, '*'),
          kind: 'cssAtImport'
        },
        {
          meta: {},
          caller: 'js-plugin-less'
        }
      );
      if (result) {
        return {
          filename: path.resolve(result.resolvedPath),
          contents:
            result.contents ?? (await fsp.readFile(result.resolved, 'utf-8'))
        };
      } else {
        return super.loadFile(filename, dir, opts, env);
      }
    }
  };

  return {
    install(_, pluginManager) {
      pluginManager.addFileManager(new CustomLessManager(resolvedPath));
    },
    minVersion: [3, 0, 0]
  };
}
