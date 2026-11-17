import assert from "node:assert/strict";
import { execFile, spawnSync } from "node:child_process";
import fs from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import { promisify } from "node:util";
import test from "node:test";
import { assertDiagnosticFailure } from "./e2e-diagnostics.mjs";

const execFileAsync = promisify(execFile);
const base = new URL("./", import.meta.url);
async function fixture(t, prefix, files) {
  const root = await fs.mkdtemp(path.join(os.tmpdir(), prefix));
  t.after(() => fs.rm(root, { recursive: true, force: true }));
  for (const file of files)
    await fs.copyFile(new URL(file, base), path.join(root, file));
  return root;
}

test("diagnostic rejection requires a completed nonzero exit, never timeout or signal", () => {
  const diagnostic = "static dependency cycle";
  const result = spawnSync(process.execPath, ["-e", `console.error(${JSON.stringify(diagnostic)});setInterval(()=>{},1000);`], { encoding: "utf8", timeout: 2000 });
  assert.match(result.stderr, /static dependency cycle/);
  assert.equal(result.error?.code, "ETIMEDOUT");
  assert.throws(() => assertDiagnosticFailure(result, /static dependency cycle/), { code: "ETIMEDOUT" });
  const normal = spawnSync(process.execPath, ["-e", `console.error(${JSON.stringify(diagnostic)});process.exitCode=1;`], { encoding: "utf8", timeout: 5000 });
  assert.match(assertDiagnosticFailure(normal, /static dependency cycle/), /static dependency cycle/);
  for (const invalid of [{ status: null }, { signal: "SIGTERM" }, { status: 0 }])
    assert.throws(() => assertDiagnosticFailure({ ...normal, ...invalid }, /static dependency cycle/));
  assert.throws(() => assertDiagnosticFailure({ ...normal, stderr: "unrelated failure" }, /static dependency cycle/));
});

test("failed owned POSIX stop is retryable and concurrent attempts stay shared", { skip: process.platform === "win32" }, async () => {
  const code = `
import assert from "node:assert/strict";
import cp from "node:child_process";
import { EventEmitter } from "node:events";
import { syncBuiltinESMExports } from "node:module";
const child = Object.assign(new EventEmitter(), {pid:43210,exitCode:null,signalCode:null});
cp.spawn = () => child;
syncBuiltinESMExports();
let fail = true;
const calls=[];
process.kill = (pid,signal) => {
  calls.push({pid,signal});
  if(fail) throw Object.assign(new Error("injected permission failure"),{code:"EPERM"});
  child.signalCode="SIGTERM"; child.emit("exit",null,"SIGTERM");
};
const {spawnOwnedProcess,stopOwnedProcess}=await import(${JSON.stringify(new URL("e2e-processes.mjs",base).href)});
const owned=spawnOwnedProcess("fixture",[]);
const first=stopOwnedProcess(owned);
assert.equal(stopOwnedProcess(owned),first);
await assert.rejects(first,{code:"EPERM"});
fail=false;
const retry=stopOwnedProcess(owned);
assert.notEqual(retry,first);
assert.equal(stopOwnedProcess(owned),retry);
await retry;
await stopOwnedProcess(owned);
assert.deepEqual(calls,[{pid:-43210,signal:"SIGTERM"},{pid:-43210,signal:"SIGTERM"},{pid:-43210,signal:"SIGKILL"}]);
`;
  await execFileAsync(process.execPath, ["--input-type=module", "-e", code], { timeout: 5000 });
});

for (const mode of ["delayed-success", "dev-timeout", "startup-timeout"]) {
  test(`cleanup check uses separate startup/dev clocks: ${mode}`, async (t) => {
    const root = await fixture(t, "farm-mf-clock-", ["e2e-cleanup-check.mjs"]);
    await fs.writeFile(path.join(root,"e2e-processes.mjs"), `
import {EventEmitter} from "node:events";
const child=Object.assign(new EventEmitter(),{exitCode:null,signalCode:null});
export const spawnOwnedProcess=()=>child;
export const shutdownOwnedSuite=async()=>{child.exitCode=130;return {code:130,signal:null}};
export const stopOwnedProcess=async()=>{};
`);
    await fs.writeFile(path.join(root,"runner.mjs"), `
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import net from "node:net";
import {EventEmitter} from "node:events";
const mode=${JSON.stringify(mode)};
let clock=0, evidence, fetches=0;
Date.now=()=>clock;
globalThis.setTimeout=(callback)=>{clock+=1000;queueMicrotask(callback)};
fs.readFile=async()=>Buffer.from("original");
fs.open=async()=>({fd:1,close:async()=>{}});
fs.writeFile=async(_file,value)=>{evidence=JSON.parse(value)};
fs.stat=async(file)=>{
  if(String(file).endsWith(".e2e-running"))throw Object.assign(new Error("no lock"),{code:"ENOENT"});
  if(clock<30000 || mode==="startup-timeout")throw Object.assign(new Error("no stage"),{code:"ENOENT"});
  return {mtimeMs:30000};
};
globalThis.fetch=async()=>{fetches++;return {ok:mode==="delayed-success" && clock>=40000,arrayBuffer:async()=>new ArrayBuffer(0)}};
net.createServer=()=>Object.assign(new EventEmitter(),{listen(_port,_host,callback){callback()},close(callback){callback()}});
await import("./e2e-cleanup-check.mjs");
assert.equal(evidence.passed,mode==="delayed-success");
if(mode==="delayed-success"){assert.equal(clock,40000);assert.ok(fetches>0)}
if(mode==="dev-timeout"){assert.equal(clock,50000);assert.ok(fetches>0)}
if(mode==="startup-timeout"){assert.equal(clock,380000);assert.equal(fetches,0)}
process.exitCode=0;
`);
    await execFileAsync(process.execPath, ["runner.mjs"], {cwd:root,timeout:5000});
  });
}

for(const interrupt of [false,true]){
  test(`snapshot initialization ${interrupt ? "interruption" : "failure"} releases lock and listeners`, async(t)=>{
    const root=await fixture(t,"farm-mf-lock-",["e2e-suite.mjs","e2e-processes.mjs"]);
    const fixtures=["remote-a/lazy.js","remote-a/Button.js","remote-b/Button.js","remote-a/button.css"];
    for(const file of fixtures){await fs.mkdir(path.dirname(path.join(root,file)),{recursive:true});await fs.writeFile(path.join(root,file),"original\n")}
    await fs.writeFile(path.join(root,"runner.mjs"),`
import assert from "node:assert/strict";
import fs from "node:fs/promises";
import net from "node:net";
import {EventEmitter} from "node:events";
// This fixture checks initialization, not port ownership. Avoid touching live
// ports or racing the separate real-process cleanup tests in another file.
net.createServer=()=>Object.assign(new EventEmitter(),{listen(_port,_host,callback){callback()},close(callback){callback()}});
const before=["message","SIGINT","SIGTERM"].map(k=>process.listenerCount(k));
const read=fs.readFile.bind(fs), open=fs.open.bind(fs);
let closes=0, injected=false, exitCode;
fs.open=async(...args)=>{const file=await open(...args);if(String(args[0]).endsWith(".e2e-running")){const close=file.close.bind(file);file.close=async()=>{closes++;await close()}};return file};
fs.readFile=async(...args)=>{
 if(!injected && String(args[0]).endsWith("remote-a/lazy.js")){
  injected=true;
  ${interrupt ? 'process.emit("message",{type:"mf:e2e:shutdown"});await new Promise(resolve=>setImmediate(resolve));' : 'throw Object.assign(new Error("injected snapshot read failure"),{code:"EIO"});'}
 }
 return read(...args);
};
process.exit=code=>{exitCode=code};
const {runSuite}=await import("./e2e-suite.mjs");
let launches=0;
await assert.rejects(runSuite({test:async()=>{launches++}}),/${interrupt ? 'shutting down' : 'injected snapshot read failure'}/);
await new Promise(resolve=>setImmediate(resolve));
assert.equal(launches,0);
assert.equal(closes,1);
${interrupt ? 'assert.equal(exitCode,130);' : ''}
await assert.rejects(fs.stat(".e2e-running"),{code:"ENOENT"});
assert.deepEqual(["message","SIGINT","SIGTERM"].map(k=>process.listenerCount(k)),before);
// A later suite can acquire the same lock; the original failure is not sticky.
fs.readFile=read;
await assert.rejects(runSuite({test:async()=>{throw new Error("retry entered tests")}}),/retry entered tests/);
assert.equal(closes,2);
await assert.rejects(fs.stat(".e2e-running"),{code:"ENOENT"});
`);
    await execFileAsync(process.execPath,["runner.mjs"],{cwd:root,timeout:5000});
  });
}
