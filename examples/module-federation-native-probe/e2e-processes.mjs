import { execFile, spawn } from "node:child_process";
import { promisify } from "node:util";

const execFileAsync = promisify(execFile);
const owned = new WeakMap();
export const suiteShutdownMessage = "mf:e2e:shutdown";
const running = (child) => child.exitCode === null && child.signalCode === null;

// Only children created here may be terminated. Never search by name or port.
export function spawnOwnedProcess(command, args, options) {
  const child = spawn(command, args, {
    ...options,
    detached: process.platform !== "win32",
  });
  owned.set(child, {});
  return child;
}

function requireOwned(child) {
  const state = owned.get(child);
  if (!state) throw new Error("Refusing to terminate an unowned process");
  return state;
}

function waitForExit(child, timeoutMs) {
  if (!running(child) || !child.pid) return Promise.resolve(true);
  return new Promise((resolve) => {
    let timer;
    const done = (exited) => {
      clearTimeout(timer);
      child.removeListener("exit", onExit);
      resolve(exited);
    };
    const onExit = () => done(true);
    child.once("exit", onExit);
    timer = setTimeout(() => done(false), timeoutMs);
  });
}

export function stopOwnedProcess(child) {
  const state = requireOwned(child);
  return state.stopping ??= (async () => {
    const pid = child.pid;
    if (!Number.isInteger(pid) || pid <= 1) return;
    if (process.platform === "win32") {
      // Windows has no POSIX process groups. Scope taskkill to this live child
      // and its descendants, never /IM or a system-wide process sweep.
      // An exited leader can leave orphans, but its PID may already be reused.
      // Do not target it: Windows crash/orphan cleanup needs a retained
      // supervisor or Job Object and is not guaranteed by this helper.
      if (!running(child)) return;
      try {
        await execFileAsync("taskkill.exe", ["/PID", String(pid), "/T", "/F"], {
          windowsHide: true,
        });
      } catch (error) {
        if (running(child)) throw error;
      }
    } else {
      const signal = (kind) => {
        try {
          process.kill(-pid, kind);
        } catch (error) {
          if (error.code !== "ESRCH") throw error;
        }
      };
      signal("SIGTERM");
      await waitForExit(child, 1500);
      // The leader may have exited while a descendant ignored SIGTERM.
      signal("SIGKILL");
    }
    if (!(await waitForExit(child, 1500)))
      throw new Error(`Owned process ${pid} did not exit after termination`);
  })();
}

// child.kill('SIGTERM') on Windows bypasses Node's handlers. Use the IPC
// channel to let the suite restore edited files and release its lock first.
export async function shutdownOwnedSuite(child, timeoutMs = 5000) {
  requireOwned(child);
  if (running(child)) {
    if (process.platform === "win32") {
      if (!child.connected)
        throw new Error("Suite shutdown requires a connected IPC channel");
      await new Promise((resolve, reject) => {
        child.send({ type: suiteShutdownMessage }, (error) => {
          if (error) reject(error);
          else resolve();
        });
      });
    } else {
      child.kill("SIGTERM");
    }
    if (!(await waitForExit(child, timeoutMs)))
      throw new Error("Suite cleanup exceeded its shutdown timeout");
  }
  return { code: child.exitCode, signal: child.signalCode };
}
