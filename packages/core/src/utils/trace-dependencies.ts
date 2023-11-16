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
      sourcemap: false,
      presetEnv: false,
      persistentCache: false,
      minify: false,
      lazyCompilation: false
    },
    jsPlugins: [],
    rustPlugins: []
  });

  return compiler;
}

export async function traceDependencies(configFilePath: string) {
  const compiler = createCompiler(configFilePath);
  const files = await compiler.traceDependencies();

  return files;
}

export async function traceDependenciesHash(configFilePath: string) {
  const compiler = createCompiler(configFilePath);
  const files = await compiler.traceDependenciesHash();

  return files;
}
