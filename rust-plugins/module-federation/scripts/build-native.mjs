import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { nativeBuildOptions } from "./native-platform.mjs";

const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const options = nativeBuildOptions(process.argv.slice(2));
const child = spawn(
  "cargo",
  [
    "build",
    "-p",
    "farmfe_plugin_module_federation",
    "--profile",
    options.profile,
    "--target",
    options.target,
    "--message-format=json",
    ...options.cargoArgs,
  ],
  {
    cwd: path.resolve(packageRoot, "../.."),
    stdio: ["inherit", "pipe", "inherit"],
  },
);
let pending = "",
  artifact;
const line = (value) => {
  let message;
  try {
    message = JSON.parse(value);
  } catch {
    return;
  }
  if (
    message.reason === "compiler-artifact" &&
    message.target?.name === "farmfe_plugin_module_federation"
  ) {
    const library = message.filenames?.find((file) => file.endsWith(options.extension));
    if (library) artifact = library;
  }
  if (message.reason === "compiler-message" && message.message?.rendered)
    process.stderr.write(message.message.rendered);
};
child.stdout.setEncoding("utf8");
child.stdout.on("data", (chunk) => {
  pending += chunk;
  let end;
  while ((end = pending.indexOf("\n")) >= 0) {
    line(pending.slice(0, end));
    pending = pending.slice(end + 1);
  }
});
await new Promise((resolve, reject) => {
  child.once("error", reject);
  // close follows stdout draining; exit can precede the final artifact message.
  child.once("close", (code, signal) =>
    code === 0 ? resolve() : reject(new Error(`cargo exited ${code ?? signal}`)),
  );
});
if (pending) line(pending);
if (!artifact)
  throw new Error(`Cargo did not emit a Federation dynamic library for ${options.target}`);
const destination = path.join(packageRoot, "npm", options.abi, "index.farm");
await fs.mkdir(path.dirname(destination), { recursive: true });
await fs.copyFile(artifact, destination);
console.log(`Native Federation plugin (${options.profile}): ${destination}`);
