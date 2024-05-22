import { Compiler } from '../compiler/index.js';
import { Config } from '../types/binding.js';
import { Logger } from './logger.js';

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
    const compiler = createTraceDepCompiler(configFilePath, logger);
    const files = await compiler.traceDependencies();
    return files;
  } catch (error) {
    console.error('Error tracing dependencies:', error);
    throw error;
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
