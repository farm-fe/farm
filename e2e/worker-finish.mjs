/** Finish only after the caller has completed browser/process cleanup. */
export function normalizeWorkerFailure(value) {
  return value instanceof Error ? value : new Error('Worker failed with a non-Error rejection',{cause:value});
}
export async function finishWorker({flushLogs, send, disconnect, exit}, ...failure) {
  await flushLogs();
  const failed=failure.length>0;
  const message=failed ? {type:'fatal',error:normalizeWorkerFailure(failure[0]).message} : {type:'done'};
  await new Promise((resolve,reject)=>send(message,(failure)=>failure ? reject(failure) : resolve()));
  disconnect();
  // The owned worker has no further work. Do not wait for unrelated timers
  // until the parent's failure-only shutdown watchdog terminates it.
  exit(failed ? 1 : 0);
}
