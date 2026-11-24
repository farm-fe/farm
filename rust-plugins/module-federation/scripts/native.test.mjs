import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import test from "node:test";

const scripts = fileURLToPath(new URL("./", import.meta.url));
const targets = [
  ["aarch64-apple-darwin", "darwin-arm64", ".dylib"],
  ["x86_64-unknown-linux-gnu", "linux-x64-gnu", ".so"],
  ["x86_64-pc-windows-msvc", "win32-x64-msvc", ".dll"],
];

async function fixture(t) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), "farm-mf-native-"));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  const packageRoot = path.join(root, "rust-plugins/module-federation");
  await fs.cp(scripts, path.join(packageRoot, "scripts"), { recursive: true });
  const preload = path.join(root, "mock-cargo.mjs");
  await fs.writeFile(preload, `
import childProcess from "node:child_process";
import { syncBuiltinESMExports } from "node:module";
import { EventEmitter } from "node:events";
import { PassThrough } from "node:stream";
import fs from "node:fs";
childProcess.spawn = (command, args, options) => {
  if (command !== "cargo") throw new Error("Unexpected command: " + command);
  fs.writeFileSync(process.env.CAPTURE, JSON.stringify({ command, args, cwd: options.cwd }));
  const child = new EventEmitter();
  child.stdout = new PassThrough();
  setImmediate(() => {
    const message = JSON.stringify({ reason: "compiler-artifact", package_id: "fixture-id", target: { name: "farmfe_plugin_module_federation", kind: ["cdylib"], crate_types: ["cdylib"] }, features: ["fixture"], fresh: false, filenames: [process.env.ARTIFACT] });
    // A process can exit before its stdout has drained. The builder must await close.
    child.emit("exit", Number(process.env.CARGO_EXIT || 0), null);
    child.stdout.on("end", () => child.emit("close", Number(process.env.CARGO_EXIT || 0), null));
    setTimeout(() => child.stdout.end(message), 10);
  });
  return child;
};
syncBuiltinESMExports();
`);
  const capture = path.join(root, "cargo.json");
  return {
    root,
    packageRoot,
    capture,
    async build(args, extension = ".dylib", env = {}) {
      const artifact = path.join(root, `libfarmfe_plugin_module_federation${extension}`);
      await fs.writeFile(artifact, "fixture native payload");
      return spawnSync(process.execPath, ["--import", preload, path.join(packageRoot, "scripts/build-native.mjs"), ...args], {
        cwd: root,
        encoding: "utf8",
        timeout: 10000,
        env: { ...process.env, CARGO_BUILD_TARGET: "", CAPTURE: capture, ARTIFACT: artifact, ...env },
      });
    },
  };
}

for (const [target, abi, extension] of targets) {
  test(`builds ${target} with one explicit profile and ABI-specific destination`, async (t) => {
    const f = await fixture(t);
    const result = await f.build(["--profile", "release-publish", "--target", target, "--abi", abi, "--locked"], extension);
    assert.equal(result.status, 0, result.stderr);
    const { args, cwd } = JSON.parse(await fs.readFile(f.capture, "utf8"));
    assert.deepEqual(args, ["build", "-p", "farmfe_plugin_module_federation", "--profile", "release-publish", "--target", target, "--message-format=json", "--locked"]);
    assert.equal(cwd, await fs.realpath(f.root));
    assert.equal(await fs.readFile(path.join(f.packageRoot, "npm", abi, "index.farm"), "utf8"), "fixture native payload");
  });
}

test("optional compiler evidence is captured only for a successful Cargo process", async t => {
  const f = await fixture(t);
  const events = path.join(f.root, "events.json");
  const env = { FARM_MF_BUILD_EVENTS: events };
  const args = ["--target", targets[0][0]];
  assert.notEqual((await f.build(args, ".dylib", { ...env, CARGO_EXIT: "17" })).status, 0);
  await assert.rejects(fs.stat(events), { code: "ENOENT" });
  const result = await f.build(args, ".dylib", env);
  assert.equal(result.status, 0, result.stderr);
  assert.deepEqual(JSON.parse(await fs.readFile(events, "utf8")), [{
    packageId: "fixture-id", target: "farmfe_plugin_module_federation",
    kinds: ["cdylib"], crateTypes: ["cdylib"], features: ["fixture"], fresh: false,
  }]);
});

test("keeps ci as the local default and honors CARGO_BUILD_TARGET", async (t) => {
  const f = await fixture(t);
  const result = await f.build(["--locked"], ".so", { CARGO_BUILD_TARGET: targets[1][0] });
  assert.equal(result.status, 0, result.stderr);
  const { args } = JSON.parse(await fs.readFile(f.capture, "utf8"));
  assert.deepEqual(args.slice(3, 8), ["--profile", "ci", "--target", targets[1][0], "--message-format=json"]);
  assert.equal(await fs.readFile(path.join(f.packageRoot, "npm/linux-x64-gnu/index.farm"), "utf8"), "fixture native payload");
});

test("supports equals options without leaking --abi to cargo", async (t) => {
  const f = await fixture(t);
  const result = await f.build(["--profile=release-publish", "--target=x86_64-pc-windows-msvc", "--abi=win32-x64-msvc", "--target-dir=out"], ".dll");
  assert.equal(result.status, 0, result.stderr);
  const { args } = JSON.parse(await fs.readFile(f.capture, "utf8"));
  assert.ok(args.includes("--target-dir=out"));
  assert.ok(!args.some((arg) => arg.startsWith("--abi")));
});

for (const args of [
  ["--target", targets[1][0], "--abi", targets[2][1]],
  ["--target", "wasm32-unknown-unknown"],
  ["--target"],
  ["--profile", "--locked"],
  ["--profile", "ci", "--profile", "release-publish"],
  ["--message-format=short"],
]) {
  test(`rejects invalid options before cargo: ${args.join(" ")}`, async (t) => {
    const f = await fixture(t);
    const result = await f.build(args);
    assert.notEqual(result.status, 0);
    await assert.rejects(fs.stat(f.capture), { code: "ENOENT" });
  });
}

test("never stages a failed cargo build", async (t) => {
  const f = await fixture(t);
  const result = await f.build(["--target", targets[0][0]], ".dylib", { CARGO_EXIT: "17" });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /cargo exited 17/);
  await assert.rejects(fs.stat(path.join(f.packageRoot, "npm")), { code: "ENOENT" });
});

for (const [, abi] of targets) {
  test(`loader selects the same ${abi} path as the builder`, () => {
    const [platform, arch] = abi.split("-");
    const source = `
Object.defineProperty(process, "platform", { value: ${JSON.stringify(platform)} });
Object.defineProperty(process, "arch", { value: ${JSON.stringify(arch)} });
process.report.getReport = () => ({ header: { glibcVersionRuntime: "2.39" } });
const { default: federation } = await import(${JSON.stringify(new URL("./index.mjs", import.meta.url).href)});
console.log(JSON.stringify(federation({ name: "fixture" })));
`;
    const result = spawnSync(process.execPath, ["--input-type=module", "-e", source], { encoding: "utf8", timeout: 10000 });
    assert.equal(result.status, 0, result.stderr);
    const [binary, options] = JSON.parse(result.stdout);
    assert.equal(binary, path.resolve(scripts, "..", "npm", abi, "index.farm"));
    assert.equal(options.name, "fixture");
    assert.ok(path.isAbsolute(options.runtimePath));
  });
}

for (const profileArgs of [["--profile", "release"], ["--release"], ["-r"]]) {
  test(`accepts release profile syntax ${profileArgs.join(" ")}`, async (t) => {
    const f = await fixture(t);
    const result = await f.build([...profileArgs, "--target", targets[0][0]], ".dylib", { CARGO_BUILD_TARGET: "invalid-env-target" });
    assert.equal(result.status, 0, result.stderr);
    const { args } = JSON.parse(await fs.readFile(f.capture, "utf8"));
    assert.deepEqual(args.slice(3, 7), ["--profile", "release", "--target", targets[0][0]]);
  });
}

test("rejects an artifact with the wrong target's library extension", async (t) => {
  const f = await fixture(t);
  const result = await f.build(["--target", targets[1][0]], ".dylib");
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /did not emit.*x86_64-unknown-linux-gnu/);
  await assert.rejects(fs.stat(path.join(f.packageRoot, "npm")), { code: "ENOENT" });
});

test("Linux musl never silently resolves the GNU artifact", () => {
  const source = `
Object.defineProperty(process, "platform", { value: "linux" });
Object.defineProperty(process, "arch", { value: "x64" });
process.report.getReport = () => ({ header: {} });
const { default: federation } = await import(${JSON.stringify(new URL("./index.mjs", import.meta.url).href)});
federation({});
`;
  const result = spawnSync(process.execPath, ["--input-type=module", "-e", source], { encoding: "utf8", timeout: 10000 });
  assert.notEqual(result.status, 0);
  assert.match(result.stderr, /Unsupported experimental Federation platform: linux-x64-musl/);
});

test("private package metadata declares exactly the three supported native targets", async () => {
  const manifest = JSON.parse(await fs.readFile(new URL("../package.json", import.meta.url), "utf8"));
  const { nativeTarget } = await import("./native-platform.mjs");
  assert.equal(manifest.private, true, "experimental MF must remain private");
  assert.deepEqual([...manifest.napi.targets].sort(), targets.map(([target]) => target).sort());
  for (const [target, abi, extension] of targets)
    assert.deepEqual(nativeTarget(target), { target, abi, extension });
});

async function adapterShareScope(t, mode) {
  const name = "testFarmProvider";
  const namespace = "farm_native_selection_" + mode;
  globalThis[namespace] = { m: {} };
  t.after(() => { delete globalThis[namespace]; });
  const asModule = source => "data:text/javascript;base64," + Buffer.from(source).toString("base64");
  const runtime = asModule("export function createInstance(options) { return options; }");
  const source = await fs.readFile(new URL("../src/adapter.js", import.meta.url), "utf8");
  const importable = source.replace("__MF_RUNTIME_PATH__", JSON.stringify(runtime))
    .replace("__MF_CONFIGURATION__", JSON.stringify({ namespace, name, hot: false }))
    .replace("__MF_SHARED__", `Object.fromEntries(["react","unselected","non-singleton","foreign","already-loaded"].map(id => [
      id, {version:"18.3.1",shareConfig:{singleton:id!=="non-singleton"},get:() => { throw new Error("duplicate getter"); }}
    ]))`)
    .replace("__MF_HOT_ADAPTER__", "");
  const { bridge } = await import(asModule(importable));
  const { plugins, shared } = bridge.instance;
  let resolve, reject;
  const pending = new Promise((yes, no) => { resolve = yes; reject = no; });
  const entries = Object.fromEntries(Object.entries(shared).map(([id, config]) => [id, { "18.3.1": {
    from: id === "foreign" ? "anotherProvider" : name, loaded: id === "already-loaded",
    loading: id === "unselected" ? undefined : pending, get: config.get,
  }, "18.4.0": { from: name, loaded: false, loading: pending, get: config.get } }]));
  const data = { shareScope: entries };
  assert.equal(plugins[0].beforeInitContainer(data), data, "hook must not await an in-flight factory");
  const entry = entries.react["18.3.1"];
  assert.equal(entry.loaded, true);
  assert.equal(entry.get(), pending);
  assert.equal(plugins[0].beforeInitContainer(data), data);
  assert.equal(entry.get(), pending, "repeated init shares the same getter operation");
  for (const [id, version] of [["unselected", "18.3.1"], ["non-singleton", "18.3.1"],
    ["foreign", "18.3.1"], ["react", "18.4.0"], ["already-loaded", "18.3.1"]]) {
    const other = entries[id][version];
    assert.equal(other.loaded, id === "already-loaded");
    assert.equal(other.get, shared[id].get);
  }
  return { entry, priorGet: shared.react.get, pending, resolve, reject };
}

test("keeps a selected local singleton while its async factory is still in flight", async t => {
  const { entry, priorGet, pending, resolve } = await adapterShareScope(t, "success");
  const factory = () => ({ react: true });
  resolve(factory);
  assert.equal(await pending, factory);
  assert.equal(entry.get, priorGet);
  assert.equal(entry.loaded, true);
});

test("failed singleton selection releases its provisional loaded marker and getter", async t => {
  const { entry, priorGet, pending, reject } = await adapterShareScope(t, "failure");
  reject(new Error("shared chunk failed"));
  await assert.rejects(pending, /shared chunk failed/);
  assert.equal(entry.get, priorGet);
  assert.equal(entry.loaded, false);
  assert.equal(entry.loading, undefined);
});

test("a rejected older singleton does not overwrite a newer provider operation", async t => {
  const { entry, pending, reject } = await adapterShareScope(t, "replaced");
  const replacementFactory = () => ({ replacement: true });
  const replacementLoading = Promise.resolve(replacementFactory);
  const replacementGet = () => replacementLoading;
  Object.assign(entry, { loaded: true, loading: replacementLoading, lib: replacementFactory, get: replacementGet });
  reject(new Error("older shared chunk failed"));
  await assert.rejects(pending, /older shared chunk failed/);
  assert.equal(entry.loaded, true);
  assert.equal(entry.loading, replacementLoading);
  assert.equal(entry.get, replacementGet);
  assert.equal(entry.lib, replacementFactory);
});
