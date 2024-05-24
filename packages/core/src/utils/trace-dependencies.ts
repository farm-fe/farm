import { Config } from '../../binding/index.js';
import { Compiler } from '../compiler/index.js';
import { convertErrorMessage } from './error.js';

function createTraceDepCompiler(entry: string) {
  const config = getDefaultTraceDepCompilerConfig(entry);
  config.config.progress = false;
  return new Compiler(config);
}

export async function traceDependencies(
  configFilePath: string
): Promise<string[]> {
  try {
    const compiler = createTraceDepCompiler(configFilePath);
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
