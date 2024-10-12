import { convertErrorMessage } from './error';

import * as fs from 'node:fs';
import { createInlineCompiler } from '../compiler/index';
import { ResolvedUserConfig } from '../config/types';

function createTraceDepCompiler(entry: string) {
  const config = getDefaultTraceDepCompilerConfig(entry);

  return createInlineCompiler(config);
}

export async function traceDependencies(
  configFilePath: string
): Promise<string[]> {
  try {
    // maybe not find config from local
    if (
      !(fs.existsSync(configFilePath) && fs.statSync(configFilePath).isFile())
    ) {
      return [];
    }

    const compiler = createTraceDepCompiler(configFilePath);
    const files = (await compiler.traceDependencies()) as string[];
    return files;
  } catch (error) {
    const errorMessage = convertErrorMessage(error);
    throw Error(`Error tracing dependencies: ${errorMessage}`);
  }
}

function getDefaultTraceDepCompilerConfig(entry: string): ResolvedUserConfig {
  return {
    compilation: {
      input: {
        index: entry
      },
      resolve: {
        autoExternalFailedResolve: true
      },
      external: ['^[^./].*'],
      sourcemap: false,
      presetEnv: false,
      persistentCache: false,
      minify: false,
      progress: false,
      lazyCompilation: false
    },
    jsPlugins: [
      {
        name: 'trace-dependencies-ignore-node-file-plugin',
        load: {
          filters: {
            resolvedPaths: ['\\.node$']
          },
          executor: () => {
            return {
              content: '',
              moduleType: 'js'
            };
          }
        }
      }
    ],
    rustPlugins: []
  };
}
