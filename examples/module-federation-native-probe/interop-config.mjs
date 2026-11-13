import path from "node:path";

export function selectRemoteA(configs, paths = path) {
  const config = configs.find((entry) => paths.basename(entry.root) === "remote-a");
  if (!config) throw new Error("Missing remote-a interop configuration");
  return config;
}
