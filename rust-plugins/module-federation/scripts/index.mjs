import { fileURLToPath } from "node:url";

// Packaging bridge only: all graph integration and entry generation is Rust.
export default function federation(options) {
  return [
    fileURLToPath(
      new URL(
        `../npm/${process.platform}-${process.arch}/index.farm`,
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
