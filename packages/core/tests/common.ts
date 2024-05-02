import { fileURLToPath } from "node:url";
import path from "path";
import { Compiler } from "../src/compiler/index.js";
import { normalizeUserCompilationConfig } from "../src/config/index.js";
import { Logger } from "../src/index.js";
import type { JsPlugin } from "../src/plugin/type.js";

export async function getCompiler(
  root: string,
  p: string,
  plugins: JsPlugin[],
  input?: Record<string, string>,
  output?: Record<string, string>
): Promise<Compiler> {
  const originalExit = process.exit;
  process.exit = (code) => {
    console.trace("call process.exit when test");
    return originalExit(code);
  };

  const compilationConfig = await normalizeUserCompilationConfig(
    {
      root,
      compilation: {
        input: input ?? {
          index: "./index.ts?foo=bar" // Farm does not recommand using query strings in input. We just use it for testing.
        },
        output: {
          path: path.join("dist", p),
          entryFilename: "[entryName].mjs",
          targetEnv: "node",
          ...(output ?? {})
        },
        progress: false,
        lazyCompilation: false,
        sourcemap: false,
        persistentCache: false
      },
      plugins
    },
    new Logger(),
    "production"
  );

  return new Compiler({
    config: compilationConfig,
    jsPlugins: plugins,
    rustPlugins: []
  });
}

export function getFixturesDir() {
  const currentDir = path.dirname(fileURLToPath(import.meta.url));
  return path.resolve(currentDir, "fixtures");
}

export function getOutputFilePath(root: string, p: string) {
  return path.join(root, "dist", p, "index.mjs");
}
