import {
  spawnOwnedProcess,
  shutdownOwnedSuite,
  stopOwnedProcess,
} from "./e2e-processes.mjs";
import fs from "node:fs/promises";
import net from "node:net";
import path from "node:path";
import { fileURLToPath } from "node:url";
import assert from "node:assert/strict";
const base = path.dirname(fileURLToPath(import.meta.url));
const original = await fs.readFile(path.join(base, "remote-a/lazy.js"));
const started = Date.now();
const log = await fs.open(path.join(base, "cleanup-check.log"), "w");
const child = spawnOwnedProcess(process.execPath, ["e2e.mjs"], {
  cwd: base,
  env: process.env,
  stdio: ["ignore", log.fd, log.fd, "ipc"],
});
let spawnError;
child.on("error", (error) => { spawnError = error; });
await log.close();
const evidence = { passed: false };
try {
  let ready = false;
  while (Date.now() - started < 20000) {
    if (spawnError) throw spawnError;
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
  evidence.exit = await shutdownOwnedSuite(child);
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
  try {
    if (child.exitCode === null && child.signalCode === null)
      await shutdownOwnedSuite(child);
  } catch (error) {
    evidence.error ??= error.stack;
    evidence.passed = false;
    process.exitCode = 1;
  } finally {
    try {
      await stopOwnedProcess(child);
    } catch (error) {
      evidence.error ??= error.stack;
      evidence.passed = false;
      process.exitCode = 1;
    }
  }
  await fs.writeFile(
    path.join(base, "evidence-e2e-cleanup.json"),
    JSON.stringify(evidence, null, 2),
  );
  console.log(JSON.stringify(evidence, null, 2));
}
