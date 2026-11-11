import { spawn } from "node:child_process";
import fs from "node:fs/promises";
import path from "node:path";
import { fileURLToPath } from "node:url";
import net from "node:net";
const base = path.dirname(fileURLToPath(import.meta.url));
const logs = path.join(base, "e2e-results");
const active = new Set();
async function stop(child) {
  if (!child) return;
  const signal = (kind) => {
    try {
      process.kill(-child.pid, kind);
    } catch (e) {
      if (e.code !== "ESRCH") throw e;
    }
  };
  signal("SIGTERM");
  if (child.exitCode === null && child.signalCode === null)
    await Promise.race([
      new Promise((resolve) => child.once("exit", resolve)),
      new Promise((resolve) => setTimeout(resolve, 1500)),
    ]);
  signal("SIGKILL");
  active.delete(child);
}
async function launch(script, env, label) {
  const file = await fs.open(path.join(logs, label + ".log"), "w");
  const child = spawn(process.execPath, [script], {
    cwd: base,
    env: {
      ...process.env,
      MF_FORMAT: "global",
      MF_SERVE: "static",
      MF_MODE: "production",
      MF_CSP: "",
      MF_BUNDLER: "rspack",
      MF_HMR: "",
      ...env,
    },
    detached: true,
    stdio: ["ignore", file.fd, file.fd],
  });
  active.add(child);
  child.completion = new Promise((resolve, reject) => {
    child.once("error", reject);
    child.once("exit", (code, signal) =>
      code === 0
        ? resolve()
        : reject(
            new Error(
              `${label} failed (${code ?? signal}); see ${path.join(logs, label + ".log")}`,
            ),
          ),
    );
  });
  child.completion.catch(() => {});
  await file.close();
  return child;
}
async function run(script, env, label) {
  const child = await launch(script, env, label);
  let timer;
  try {
    await Promise.race([
      child.completion,
      new Promise((_, reject) => {
        timer = setTimeout(
          () => reject(new Error(`${label} exceeded 120 seconds`)),
          120000,
        );
      }),
    ]);
  } finally {
    clearTimeout(timer);
    await stop(child);
  }
}
async function readiness(child) {
  const deadline = Date.now() + 20000;
  while (Date.now() < deadline) {
    if (child.exitCode !== null || child.signalCode !== null)
      throw new Error("dev servers exited before readiness");
    try {
      const responses = await Promise.all(
        [
          "http://127.0.0.1:4100/",
          "http://127.0.0.1:4101/deploy/a/remoteEntry.js",
          "http://127.0.0.1:4102/deploy/b/remoteEntry.js",
        ].map((url) => fetch(url, { signal: AbortSignal.timeout(1500) })),
      );
      if (responses.every((r) => r.ok)) return;
    } catch {}
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("dev server readiness timeout");
}
async function requireFreePorts() {
  for (const port of [4100, 4101, 4102, 4190, 4200, 4201, 4202]) {
    const server = net.createServer();
    await new Promise((resolve, reject) => {
      server.once("error", reject);
      server.listen(port, "127.0.0.1", resolve);
    });
    await new Promise((resolve) => server.close(resolve));
  }
}
export async function runSuite(ctx) {
  const results = [];
  const test = (name, fn) =>
    ctx.test(name, async () => {
      const started = Date.now();
      try {
        await fn();
        results.push({ name, passed: true, durationMs: Date.now() - started });
      } catch (error) {
        results.push({
          name,
          passed: false,
          durationMs: Date.now() - started,
          error: error.stack,
        });
        throw error;
      }
    });
  await fs.mkdir(logs, { recursive: true });
  const lockPath = path.join(base, ".e2e-running");
  const lock = await fs.open(lockPath, "wx");
  const originals = await Promise.all(
    [
      "remote-a/lazy.js",
      "remote-a/Button.js",
      "remote-b/Button.js",
      "remote-a/button.css",
    ].map(async (file) => {
      const full = path.join(base, file);
      return [full, await fs.readFile(full)];
    }),
  );
  let cleaning;
  const cleanup = () =>
    (cleaning ??= (async () => {
      await Promise.all([...active].map(stop));
      for (const [file, original] of originals)
        if (!(await fs.readFile(file)).equals(original))
          await fs.writeFile(file, original);
      await lock.close();
      await fs.unlink(lockPath);
    })());
  const terminate = () => {
    void cleanup().finally(() => process.exit(130));
  };
  process.once("SIGINT", terminate);
  process.once("SIGTERM", terminate);
  try {
    await requireFreePorts();
    await test("non-MF CJS/ESM and async diagnostics", () =>
      run("core-regression.mjs", {}, "core-regression"));
    for (const format of ["global", "esm"]) {
      const env = { MF_FORMAT: format };
      await test(`preview ${format}: two remotes, rendered React, resources, cycles, failures and reload retry`, async () => {
        await run("build.mjs", env, format + "-build");
        await run("browser.mjs", { ...env, MF_CSP: "1" }, format + "-preview");
      });
      await test(`start ${format}: real dev servers and remote edit/full reload`, async () => {
        const child = await launch("dev.mjs", env, format + "-dev-server");
        try {
          await readiness(child);
          await run(
            "browser.mjs",
            { ...env, MF_SERVE: "dev", MF_MODE: "development" },
            format + "-dev",
          );
          await run("refresh.mjs", env, format + "-refresh");
        } finally {
          await stop(child);
        }
      });
      await test(`hot ${format}: remote JS/CSS, preserved state, rollback and connection recovery`, async () => {
        const hotEnv = { ...env, MF_HMR: "1" };
        const child = await launch("dev.mjs", hotEnv, format + "-hot-server");
        try {
          await readiness(child);
          await run("hmr-browser.mjs", hotEnv, format + "-hot-browser");
        } finally {
          await stop(child);
        }
      });
      for (const bundler of ["rspack", "webpack"])
        await test(`${bundler} ↔ Farm ${format}: shared identity, CSS and chunks`, async () => {
          const interop = { ...env, MF_BUNDLER: bundler };
          await run(
            "interop-build.mjs",
            interop,
            bundler + "-" + format + "-build",
          );
          await run(
            "interop-browser.mjs",
            interop,
            bundler + "-" + format + "-browser",
          );
        });
    }
  } finally {
    await cleanup();
    process.removeListener("SIGINT", terminate);
    process.removeListener("SIGTERM", terminate);
    await fs.writeFile(
      path.join(base, "evidence-suite.json"),
      JSON.stringify(
        {
          passed: results.length === 11 && results.every((r) => r.passed),
          results,
        },
        null,
        2,
      ),
    );
  }
}
