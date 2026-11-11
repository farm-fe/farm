import assert from "node:assert/strict";
import { execFile, spawn } from "node:child_process";
import fs from "node:fs/promises";
import net from "node:net";
import os from "node:os";
import path from "node:path";
import { setTimeout as delay } from "node:timers/promises";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";
import test from "node:test";

const base = fileURLToPath(new URL("./", import.meta.url));
const execFileAsync = promisify(execFile);
const alive = (child) => child.exitCode === null && child.signalCode === null;
async function until(check, timeout = 5000) {
  const end = Date.now() + timeout;
  while (Date.now() < end) {
    if (await check()) return;
    await delay(20);
  }
  assert.fail("Timed out waiting for owned fixture processes");
}
async function freePort(port) {
  const server = net.createServer();
  await new Promise((resolve, reject) => {
    server.once("error", reject);
    server.listen(port, "127.0.0.1", resolve);
  });
  await new Promise((resolve) => server.close(resolve));
}

for (const interruption of ["ipc", ...(process.platform === "win32" ? [] : ["signal"])]) {
  test(`real suite ${interruption} shutdown closes owned children and restores source/lock`, { timeout: 15000 }, async (t) => {
    const root = await fs.mkdtemp(path.join(os.tmpdir(), "farm-mf-cleanup-"));
    const fixtureFiles = ["remote-a/lazy.js", "remote-a/Button.js", "remote-b/Button.js", "remote-a/button.css"];
    let suite;
    t.after(async () => {
      if (suite && alive(suite)) {
        if (process.platform === "win32") {
          await execFileAsync("taskkill.exe", ["/PID", String(suite.pid), "/T", "/F"]).catch(() => {});
        } else {
          suite.kill("SIGTERM");
        }
        await until(() => !alive(suite));
      }
      await fs.rm(root, { recursive: true, force: true });
    });
    for (const file of ["e2e-suite.mjs", "e2e-processes.mjs"]) {
      await fs.copyFile(path.join(base, file), path.join(root, file)).catch((error) => {
        if (file !== "e2e-processes.mjs" || error.code !== "ENOENT") throw error;
      });
    }
    for (const file of fixtureFiles) {
      await fs.mkdir(path.dirname(path.join(root, file)), { recursive: true });
      await fs.writeFile(path.join(root, file), "original\n");
    }
    await fs.writeFile(path.join(root, "runner.mjs"), `
import { appendFileSync } from "node:fs";
import { runSuite } from "./e2e-suite.mjs";
// Match SpecRunner: record a failure and return so the caller can continue.
await runSuite({ test: async (name, run) => {
  appendFileSync("started-tests.log", name + "\\n");
  try { await run(); } catch {}
} });
`);
    // The suite launches this real process. Its descendant ignores SIGTERM,
    // proving process-tree cleanup, not merely termination of the direct child.
    await fs.writeFile(path.join(root, "core-regression.mjs"), `
import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import net from "node:net";
const child = spawn(process.execPath, ["--input-type=module", "-e", ${JSON.stringify(`
import net from "node:net";
process.on("SIGTERM", () => {});
net.createServer().listen(0, "127.0.0.1", function () { process.send({ port: this.address().port }); });
`)}], { stdio: ["ignore", "ignore", "ignore", "ipc"] });
child.once("message", async ({ port }) => {
  await fs.writeFile("remote-a/lazy.js", "changed by fixture\\n");
  net.createServer().listen(0, "127.0.0.1", async function () {
    await fs.writeFile("ready.json", JSON.stringify({ ports: [port, this.address().port] }));
  });
});
`);
    // A separate listener must survive cleanup of the suite's owned tree.
    const unrelated = net.createServer();
    await new Promise((resolve) => unrelated.listen(0, "127.0.0.1", resolve));
    t.after(() => new Promise((resolve) => unrelated.close(resolve)));
    suite = spawn(process.execPath, ["runner.mjs"], {
      cwd: root,
      detached: process.platform !== "win32",
      stdio: ["ignore", "ignore", "pipe", "ipc"],
    });
    let errors = "";
    suite.stderr.on("data", (chunk) => { errors += chunk; });
    await until(async () => {
      assert.ok(alive(suite), errors);
      return fs.stat(path.join(root, "ready.json")).then(() => true, () => false);
    });
    const { ports } = JSON.parse(await fs.readFile(path.join(root, "ready.json"), "utf8"));
    if (interruption === "ipc") suite.send({ type: "mf:e2e:shutdown" });
    else suite.kill("SIGTERM");
    await until(() => !alive(suite), 3000);
    assert.equal(suite.exitCode, 130, errors);
    const startedTests = (await fs.readFile(path.join(root, "started-tests.log"), "utf8")).trim().split("\n");
    assert.equal(startedTests.length, 1, "shutdown must not start another test after the runner catches failure");
    for (const port of ports) await freePort(port);
    for (const file of fixtureFiles)
      assert.equal(await fs.readFile(path.join(root, file), "utf8"), "original\n");
    await assert.rejects(fs.stat(path.join(root, ".e2e-running")), { code: "ENOENT" });
    assert.equal(unrelated.listening, true, "must not kill unrelated listeners");
  });
}

test("Windows branch targets only owned PIDs, uses IPC, and reports taskkill failure", async () => {
  // Execute real helper code with only OS calls stubbed. Never issue a Windows
  // command, fake PID signal, or process-wide kill on the current computer.
  const source = `
import assert from "node:assert/strict";
import childProcess from "node:child_process";
import { EventEmitter } from "node:events";
import { syncBuiltinESMExports } from "node:module";
Object.defineProperty(process, "platform", { value: "win32" });
const spawned = [];
const calls = [];
let failKill = false;
childProcess.spawn = (_command, _args, options) => {
  const child = new EventEmitter();
  Object.assign(child, { pid: 43210 + spawned.length, exitCode: null, signalCode: null, connected: true, options });
  child.kill = () => { throw new Error("Windows shutdown must not bypass suite cleanup"); };
  child.send = (message, callback) => {
    child.message = message;
    child.exitCode = 130;
    child.emit("exit", 130, null);
    callback(null);
  };
  spawned.push(child);
  return child;
};
childProcess.execFile = (command, args, options, callback) => {
  calls.push({ command, args, options });
  setImmediate(() => {
    if (failKill) return callback(new Error("taskkill fixture failure"));
    const child = spawned.find((candidate) => String(candidate.pid) === args[1]);
    child.exitCode = 1;
    child.emit("exit", 1, null);
    callback(null, "", "");
  });
};
process.kill = () => { throw new Error("Windows must never send a POSIX process-group signal"); };
syncBuiltinESMExports();
const { spawnOwnedProcess, stopOwnedProcess, shutdownOwnedSuite } = await import(${JSON.stringify(new URL("./e2e-processes.mjs", import.meta.url).href)});
const child = spawnOwnedProcess("fixture", [], { detached: true });
assert.equal(child.options.detached, false);
assert.throws(() => stopOwnedProcess({ pid: child.pid }), /unowned/);
const stopped = stopOwnedProcess(child);
assert.equal(stopOwnedProcess(child), stopped, "concurrent cleanup must not terminate twice");
await stopped;
await stopOwnedProcess(child);
assert.deepEqual(calls, [{ command: "taskkill.exe", args: ["/PID", String(child.pid), "/T", "/F"], options: { windowsHide: true } }]);
const exited = spawnOwnedProcess("fixture", []);
exited.exitCode = 0;
await stopOwnedProcess(exited);
assert.equal(calls.length, 1, "never target a reaped Windows PID");
const failedSpawn = spawnOwnedProcess("fixture", []);
failedSpawn.pid = undefined;
await stopOwnedProcess(failedSpawn);
assert.equal(calls.length, 1, "never target a missing PID");
const suite = spawnOwnedProcess("fixture", [], { stdio: ["ignore", "ignore", "ignore", "ipc"] });
assert.deepEqual(await shutdownOwnedSuite(suite), { code: 130, signal: null });
assert.deepEqual(suite.message, { type: "mf:e2e:shutdown" });
assert.equal(calls.length, 1, "graceful IPC shutdown must not need taskkill");
const disconnected = spawnOwnedProcess("fixture", []);
disconnected.connected = false;
await assert.rejects(shutdownOwnedSuite(disconnected), /connected IPC/);
failKill = true;
await assert.rejects(stopOwnedProcess(spawnOwnedProcess("fixture", [])), /taskkill fixture failure/);
`;
  await execFileAsync(process.execPath, ["--input-type=module", "-e", source], { timeout: 10000 });
});

test("owned cleanup handles a real failed spawn without signaling any PID", async () => {
  const { spawnOwnedProcess, stopOwnedProcess } = await import("./e2e-processes.mjs");
  const child = spawnOwnedProcess(path.join(os.tmpdir(), "missing-farm-mf-executable"), [], { stdio: "ignore" });
  const error = await new Promise((resolve) => child.once("error", resolve));
  assert.equal(error.code, "ENOENT");
  await stopOwnedProcess(child);
  await stopOwnedProcess(child);
});

test("shutdown during async log-open prevents a child escaping the cleanup snapshot", async (t) => {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "farm-mf-launch-race-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  for (const file of ["e2e-suite.mjs", "e2e-processes.mjs"])
    await fs.copyFile(path.join(base, file), path.join(root, file));
  for (const file of ["remote-a/lazy.js", "remote-a/Button.js", "remote-b/Button.js", "remote-a/button.css"]) {
    await fs.mkdir(path.dirname(path.join(root, file)), { recursive: true });
    await fs.writeFile(path.join(root, file), "original\n");
  }
  await fs.writeFile(path.join(root, "runner.mjs"), `
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import childProcess from "node:child_process";
import { syncBuiltinESMExports } from "node:module";
let launches = 0;
childProcess.spawn = () => {
  launches++;
  throw new Error("unexpected process launch during shutdown");
};
syncBuiltinESMExports();
const open = fs.open.bind(fs);
let interrupted = false;
fs.open = async (...args) => {
  const file = await open(...args);
  if (String(args[0]).endsWith("core-regression.log")) {
    interrupted = true;
    process.emit("message", { type: "mf:e2e:shutdown" });
  }
  return file;
};
// Keep the fixture alive after cleanup so the suspended launch must resume.
let exitCode;
process.exit = (code) => { exitCode = code; };
const { runSuite } = await import("./e2e-suite.mjs");
await assert.rejects(runSuite({ test: async (_name, run) => {
  try { await run(); } catch {}
} }), /shutting down/);
assert.equal(interrupted, true);
assert.equal(exitCode, 130);
assert.equal(launches, 0, "no process may launch after cleanup takes its snapshot");
await assert.rejects(fs.stat(".e2e-running"), { code: "ENOENT" });
`);
  await execFileAsync(process.execPath, ["runner.mjs"], { cwd: root, timeout: 10000 });
});
