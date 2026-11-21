import { statSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { hostNativeTarget } from "./native-platform.mjs";

export default function federation(options) {
  const { abi } = hostNativeTarget();
  const binary = fileURLToPath(new URL(`../native/${abi}/index.farm`, import.meta.url));
  const info = statSync(binary, { throwIfNoEntry: false });
  if (!info?.isFile()) throw new Error(`Missing bundled Federation binary for ${abi}`);
  return [binary, {
    ...options,
    runtimePath: fileURLToPath(import.meta.resolve("@module-federation/runtime")),
  }];
}
