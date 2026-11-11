import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
const base = path.dirname(fileURLToPath(import.meta.url));
const original = await fs.readFile(path.join(base, "remote-a/lazy.js"));
const started = Date.now();
const log = await fs.open(path.join(base, "cleanup-check.log"), "w");
const child = spawn(process.execPath, ["e2e.mjs"], {
  cwd: base,
  env: process.env,
  detached: true,
  stdio: ["ignore", log.fd, log.fd],
});
const exited = new Promise((resolve, reject) => {
  child.once("error", reject);
  child.once("exit", (code, signal) => resolve({ code, signal }));
});
await log.close();
const evidence = { passed: false };
try {
  let ready = false;
  while (Date.now() - started < 20000) {
    if (child.exitCode !== null || child.signalCode !== null)
      throw new Error("Suite exited before interruption check");
    try {
      const stat = await fs.stat(
        path.join(base, "e2e-results/global-dev-server.log"),
      );
      if (stat.mtimeMs >= started) {
        const response = await fetch("http://127.0.0.1:4100/", {
          signal: AbortSignal.timeout(500),
        });
        await response.arrayBuffer();
        if (response.ok) {
          ready = true;
          break;
        }
      }
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 50));
  }
  assert.equal(ready, true, "Did not observe real dev server");
  child.kill("SIGTERM");
  let timer;
  try {
    evidence.exit = await Promise.race([
      exited,
      new Promise((_, reject) => {
        timer = setTimeout(
          () => reject(new Error("Signal cleanup exceeded 5 seconds")),
          5000,
        );
      }),
    ]);
  } finally {
    clearTimeout(timer);
  }
  for (const port of [4100, 4101, 4102, 4190, 4200, 4201, 4202]) {
    const server = net.createServer();
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(port, "127.0.0.1", resolve);
    });
    await new Promise((resolve) => server.close(resolve));
  }
  assert.equal(
    (await fs.readFile(path.join(base, "remote-a/lazy.js"))).equals(original),
    true,
  );
  await assert.rejects(
    fs.stat(path.join(base, ".e2e-running")),
    (error) => error.code === "ENOENT",
  );
  evidence.portsClosed = true;
  evidence.originalFileRestored = true;
  evidence.lockReleased = true;
  evidence.passed = true;
} catch (error) {
  evidence.error = error.stack;
  process.exitCode = 1;
} finally {
  if (child.exitCode === null && child.signalCode === null)
    child.kill("SIGTERM");
  await fs.writeFile(
    path.join(base, "evidence-e2e-cleanup.json"),
    JSON.stringify(evidence, null, 2),
  );
  console.log(JSON.stringify(evidence, null, 2));
}
