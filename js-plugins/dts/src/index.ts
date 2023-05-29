import { JsPlugin, UserConfig } from '@farmfe/core';
import fs from 'node:fs';
import { Project } from 'ts-morph';
import { getResolvedOptions, handleExclude, handleInclude } from './utils.js';
export default function farmDtsPlugin(
  farmDtsPluginOptions: any = {}
): JsPlugin {
  // options hooks to get farmConfig
  let farmConfig: UserConfig['compilation'];
  const resolvedOptions = getResolvedOptions(farmDtsPluginOptions);

  // const exclude = handleExclude(resolvedOptions);
  // const include = handleInclude(resolvedOptions);
  return {
    name: 'farm-dts-plugin',
    config(config: any) {
      farmConfig = config || {};
      return config;
    },
    load: {
      filters: {
        resolvedPaths: ['.ts$']
      },
      async executor(params: any, ctx: any) {
        const { resolvedPath } = params;
        const data = await fs.promises.readFile(resolvedPath, 'utf-8');

        let source = data;
        console.log(source);

        return {
          content: data,
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
        console.log(params);
        const project = new Project();
        console.log(project);
        // const sourceFile = project.createSourceFile(
        //   params.resolvedPath,
        //   params.content
        // );
        // console.log(sourceFile);

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
