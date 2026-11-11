import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import path from "node:path";
import fs from "node:fs/promises";
const packageRoot = fileURLToPath(new URL("../", import.meta.url));
const child = spawn(
  "cargo",
  [
    "build",
    "-p",
    "farmfe_plugin_module_federation",
    "--profile",
    "ci",
    "--message-format=json",
    ...process.argv.slice(2),
  ],
  {
    cwd: path.resolve(packageRoot, "../.."),
    stdio: ["inherit", "pipe", "inherit"],
  },
);
let pending = "",
  artifact;
const line = (value) => {
  try {
    const message = JSON.parse(value);
    if (
      message.reason === "compiler-artifact" &&
      message.target?.name === "farmfe_plugin_module_federation"
    )
      artifact = message.filenames.find((f) => /\.(dylib|so|dll)$/.test(f));
    if (message.reason === "compiler-message" && message.message?.rendered)
      process.stderr.write(message.message.rendered);
  } catch {}
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
  child.once("exit", (code) =>
    code === 0 ? resolve() : reject(new Error(`cargo exited ${code}`)),
  );
});
if (pending) line(pending);
if (!artifact)
  throw new Error("Cargo did not emit a Federation dynamic library");
const destination = path.join(
  packageRoot,
  "npm",
  `${process.platform}-${process.arch}`,
  "index.farm",
);
await fs.mkdir(path.dirname(destination), { recursive: true });
await fs.copyFile(artifact, destination);
console.log(`Native Federation plugin: ${destination}`);
