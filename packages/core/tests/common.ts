import path from 'path';
import { Compiler } from '../src/compiler/index.js';
import { JsPlugin } from '../src/plugin/index.js';
import { normalizeUserCompilationConfig } from '../src/config/index.js';
import { fileURLToPath } from 'node:url';

export async function getCompiler(
  root: string,
  p: string,
  plugins: JsPlugin[],
  input?: Record<string, string>
): Promise<Compiler> {
  const config = await normalizeUserCompilationConfig(
    {
      root,
      compilation: {
        input: input ?? {
          index: './index.ts?foo=bar'
        },
        output: {
          path: path.join('dist', p),
          filename: 'index.mjs',
          targetEnv: 'node'
        },
        lazyCompilation: false,
        sourcemap: false
      },
      server: {
        hmr: false
      },
      plugins
    },
    'production'
  );
  return new Compiler(config);
}

export function getFixturesDir() {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(currentDir, 'fixtures');
}

export function getOutputFilePath(root: string, p: string) {
  return path.join(root, 'dist', p, 'index.mjs');
}
