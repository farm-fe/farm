import { fileURLToPath } from 'node:url';
import path from 'path';
import { Compiler } from '../src/compiler/index.js';
import { JsPlugin } from '../src/plugin/type.js';
import {
  normalizeUserCompilationConfig,
  resolveMergedUserConfig,
  UserConfig
} from '../src/config/index.js';
import { Logger } from '../src/index.js';

export async function getCompiler(
  root: string,
  p: string,
  plugins: JsPlugin[],
  input?: Record<string, string>,
  output?: Record<string, string>
): Promise<Compiler> {
  const originalExit = process.exit;
  process.exit = (code) => {
    console.trace('call process.exit when test');
    return originalExit(code);
  };

  const userConfig: UserConfig = {
    root,
    compilation: {
      input: input ?? {
        index: './index.ts?foo=bar' // Farm does not recommand using query strings in input. We just use it for testing.
      },
      output: {
        path: path.join('dist', p),
        entryFilename: '[entryName].mjs',
        targetEnv: 'node',
        ...(output ?? {})
      },
      progress: false,
      lazyCompilation: false,
      sourcemap: false,
      persistentCache: false
    },
    plugins
  };
  const resolvedUserConfig = await resolveMergedUserConfig(
    userConfig,
    undefined,
    'production'
  );

  const compilationConfig = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    userConfig,
    new Logger(),
    'production'
  );

  return new Compiler({
    config: compilationConfig,
    jsPlugins: plugins,
    rustPlugins: []
  });
}

export function getFixturesDir() {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(currentDir, 'fixtures');
}

export function getOutputFilePath(root: string, p: string) {
  return path.join(root, 'dist', p, 'index.mjs');
}
