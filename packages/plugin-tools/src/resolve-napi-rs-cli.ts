import { createRequire } from "node:module";
import path from "node:path";

const require = createRequire(import.meta.url);

export function resolveNapiRsCli() {
  const packagePath = require.resolve("@napi-rs/cli/package.json");
  const packageJson = require(packagePath);
  const bin = packageJson.bin;

  if (typeof bin === "string") {
    return bin;
  }

  return path.join(path.dirname(packagePath), bin.napi);
}
