import { Compiler } from '../compiler/index.js';

function createCompiler(entry: string) {
  const compiler = new Compiler({
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
  });

  return compiler;
}

export async function traceDependencies(configFilePath: string) {
  const compiler = createCompiler(configFilePath);
  const files = await compiler.traceDependencies();

  return files;
}
