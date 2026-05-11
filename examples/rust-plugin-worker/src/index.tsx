import * as Comlink from "comlink";
import { createRoot } from "react-dom/client";
import { Main, WorkerCaseResult } from "./main";
import ComlinkWorkerCtor from "./worker/comlink.worker.ts?worker";
import WorkerCtor from "./worker/vue.worker.ts?worker";
import SharedWorkerCtor from "./worker/shared.worker.ts?sharedworker";
import workerUrl from "./worker/vue.worker.ts?worker&url";
import "./index.css";

const container = document.querySelector("#root");
const root = createRoot(container!);

const render = (workerResults: WorkerCaseResult[], running: boolean) => {
  root.render(<Main workerResults={workerResults} running={running} />);
};

const runBasicWorker = (
  worker: Worker,
  payload: [number, number],
): Promise<string> => {
  return new Promise((resolve, reject) => {
    const timeout = setTimeout(() => {
      worker.terminate();
      reject(new Error("worker response timeout"));
    }, 5000);

    worker.onmessage = (event) => {
      clearTimeout(timeout);
      worker.terminate();
      resolve(String(event.data));
    };

    worker.onerror = (event) => {
      clearTimeout(timeout);
      worker.terminate();
      reject(event.error ?? new Error(event.message));
    };

    worker.postMessage(payload);
  });
};

const runWorkerCases = async (): Promise<WorkerCaseResult[]> => {
  const results: WorkerCaseResult[] = [];

  const runCase = async (caseName: string, runner: () => Promise<string>) => {
    try {
      const detail = await runner();
      results.push({ caseName, status: "ok", detail });
    } catch (error) {
      results.push({
        caseName,
        status: "error",
        detail: error instanceof Error ? error.message : String(error),
      });
    }
  };

  await runCase("new URL absolute path", async () => {
    const worker = new Worker(new URL("/src/worker/vue.worker.ts", import.meta.url));
    return runBasicWorker(worker, [5, 5]);
  });

  await runCase("new URL relative path", async () => {
    const worker = new Worker(new URL("./worker/vue.worker.ts", import.meta.url));
    return runBasicWorker(worker, [2, 3]);
  });

  await runCase("query worker (?worker)", async () => {
    const worker = new WorkerCtor({ name: "query-worker" });
    return runBasicWorker(worker, [3, 7]);
  });

  await runCase("worker url export (?worker&url)", async () => {
    const response = await fetch(workerUrl);
    if (!response.ok) {
      throw new Error(`fetch failed: ${response.status}`);
    }
    return `Fetched worker url (${response.status})`;
  });

  await runCase("shared worker (?sharedworker)", async () => {
    return new Promise((resolve, reject) => {
      const shared = new SharedWorkerCtor({ name: "shared-worker-case" });
      const timeout = setTimeout(() => {
        reject(new Error("shared worker response timeout"));
      }, 5000);

      shared.port.onmessage = (event: MessageEvent<string>) => {
        clearTimeout(timeout);
        shared.port.close();
        resolve(event.data);
      };

      shared.port.onmessageerror = () => {
        clearTimeout(timeout);
        shared.port.close();
        reject(new Error("shared worker message error"));
      };

      shared.port.start();
      shared.port.postMessage([7, 8]);
    });
  });

  await runCase("comlink worker (?worker + comlink)", async () => {
    const worker = new ComlinkWorkerCtor({
      type: "module",
      name: "comlink-worker-case",
    });
    const api = Comlink.wrap<{ add(a: number, b: number): Promise<number> }>(worker);

    try {
      const value = await Promise.race([
        api.add(10, 20),
        new Promise<number>((_, reject) => {
          setTimeout(() => reject(new Error("comlink response timeout")), 5000);
        }),
      ]);

      return `Comlink add result: ${value}`;
    } finally {
      worker.terminate();
    }
  });

  return results;
};

render([], true);
runWorkerCases().then((results) => {
  render(results, false);
  const hasError = results.some((item) => item.status === "error");
  if (hasError) {
    console.error("Worker case failures", results);
  } else {
    console.log("All worker cases passed", results);
  }
});
