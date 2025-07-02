import { fileURLToPath, pathToFileURL } from 'node:url';
import path from 'path';
import { Compiler } from '../src/compiler/index.js';
import {
  UserConfig,
  normalizeUserCompilationConfig,
  resolveUserConfig
} from '../src/config/index.js';
import { JsPlugin } from '../src/plugin/type.js';
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
        showFileSize: false,
        ...(output ?? {})
      },
      progress: false,
      lazyCompilation: false,
      sourcemap: false,
      persistentCache: false
    },
    plugins
  };
  const resolvedUserConfig = await resolveUserConfig(userConfig, undefined);
  resolvedUserConfig.logger = new Logger();
  const compilationConfig = await normalizeUserCompilationConfig(
    resolvedUserConfig,
    'production'
  );

  return new Compiler({
    compilation: compilationConfig,
    jsPlugins: plugins,
    rustPlugins: []
  });
}

export function getFixturesDir() {
  const currentDir = decodeURIComponent(
    path.dirname(fileURLToPath(import.meta.url))
  );
  return path.resolve(currentDir, 'fixtures');
}

export function getOutputFilePath(root: string, p: string) {
  return path.join(root, 'dist', p, 'index.mjs');
}

export async function getOutputResult(outputFilePath: string) {
  if (process.platform === 'win32') {
    return await import(
      decodeURIComponent(pathToFileURL(outputFilePath).toString())
    );
  } else {
    return await import(outputFilePath);
  }
}
