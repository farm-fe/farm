import { fileURLToPath } from "node:url";
import { hostNativeTarget } from "./native-platform.mjs";

// Packaging bridge only: all graph integration and entry generation is Rust.
export default function federation(options) {
  return [
    fileURLToPath(
      new URL(
        `../npm/${hostNativeTarget().abi}/index.farm`,
        import.meta.url,
      ),
    ),
    {
      ...options,
      runtimePath: fileURLToPath(
        import.meta.resolve("@module-federation/runtime"),
      ),
    },
  ];
}
