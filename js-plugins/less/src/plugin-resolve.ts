import path from 'node:path';
import fs from 'fs/promises';

import type { CompilationContext } from 'farm';
import { rebaseUrls } from 'farm';
import type Less from 'less';

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
      const resolved = await ctx.resolve(
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
      if (resolved) {
        const result = await rebaseUrls(
          resolved.resolvedPath,
          this.rootPath,
          '@',
          async (url, importer) => {
            const res = await ctx.resolve(
              {
                source: url,
                importer,
                kind: 'cssUrl'
              },
              {
                meta: {},
                caller: 'js-plugin-less'
              }
            );
            return res.resolvedPath;
          }
        );
        return {
          filename: resolved.resolvedPath,
          contents:
            result.contents ??
            (await fs.readFile(resolved.resolvedPath, 'utf-8'))
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
