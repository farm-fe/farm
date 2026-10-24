export async function waitForDevServer(child, { port, timeoutMs = 600000, intervalMs = 30000,
  append = async () => {}, checkpoint = async () => {}, snapshot = async () => null }) {
  const record = { pid: child.pid, state: 'starting', startedAt: new Date().toISOString(), stdoutTail: '', stderrTail: '', diagnosticErrors: [] };
  let stdout = '', logWrite = Promise.resolve(), evidenceWrite = Promise.resolve();
  const save = () => {
    const data = structuredClone(record);
    evidenceWrite = evidenceWrite.then(() => checkpoint(data)).catch(error => record.diagnosticErrors.push(error.message));
  };
  let timer, interval, cleanup;
  try {
    await new Promise((resolve, reject) => {
      let settled = false;
      const finish = error => { if (settled) return; settled = true; cleanup(); error ? reject(error) : resolve(); };
      const capture = (kind, chunk) => {
        const text = chunk.toString(); record[`${kind}Tail`] = (record[`${kind}Tail`] + text).slice(-8192);
        logWrite = logWrite.then(() => append(chunk)); logWrite.catch(() => {});
        if (kind === 'stdout') {
          stdout += text;
          const match = /Local:\s+http:\/\/localhost:(\d+)/.exec(stdout.replace(/\u001b\[[0-9;]*m/g, ''));
          if (match) finish(Number(match[1]) === port ? undefined : new Error('Requested benchmark port unavailable'));
        }
      };
      const out = chunk => capture('stdout', chunk), err = chunk => capture('stderr', chunk);
      const closed = code => { record.exitCode = code; finish(new Error(`Dev server exited: ${code}`)); };
      const failed = error => finish(error);
      cleanup = () => {
        clearTimeout(timer); clearInterval(interval);
        child.stdout.off('data', out); child.stderr.off('data', err); child.off('close', closed); child.off('error', failed);
      };
      child.stdout.on('data', out); child.stderr.on('data', err); child.once('close', closed); child.once('error', failed);
      timer = setTimeout(() => finish(new Error('Dev server startup timed out')), timeoutMs);
      interval = setInterval(() => { record.observedAt = new Date().toISOString(); save(); }, intervalMs);
      save();
    });
    await logWrite; record.state = 'ready';
  } catch (error) {
    record.state = 'failed'; record.error = error.message;
    if (error.message.includes('startup timed out')) {
      try { record.timeoutProcess = await snapshot(child.pid); } catch (observationError) { record.snapshotError = observationError.message; }
    }
    throw error;
  } finally {
    cleanup?.(); await logWrite.catch(error => record.diagnosticErrors.push(error.message));
    await evidenceWrite;
    record.finishedAt = new Date().toISOString(); save(); await evidenceWrite;
  }
  return record;
}
