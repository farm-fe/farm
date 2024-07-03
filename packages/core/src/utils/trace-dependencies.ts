import { Compiler } from '../compiler/index.js';
import { convertErrorMessage } from './error.js';
import { Logger } from './logger.js';

import * as fs from 'node:fs';
import type { Config } from '../types/binding.js';

function createTraceDepCompiler(entry: string, logger: Logger) {
  const config = getDefaultTraceDepCompilerConfig(entry);
  config.config.progress = false;
  return new Compiler(config, logger);
}

export async function traceDependencies(
  configFilePath: string,
  logger: Logger
): Promise<string[]> {
  try {
    // maybe not find config from local
    if (
      !(fs.existsSync(configFilePath) && fs.statSync(configFilePath).isFile())
    ) {
      return [];
    }

    const compiler = createTraceDepCompiler(configFilePath, logger);
    const files = await compiler.traceDependencies();
    return files;
  } catch (error) {
    const errorMessage = convertErrorMessage(error);
    throw Error(`Error tracing dependencies: ${errorMessage}`);
  }
}

function getDefaultTraceDepCompilerConfig(entry: string): Config {
  return {
    config: {
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
