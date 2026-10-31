/** A crashed/incomplete worker must never turn partial successes into a green run. */
export function completeWorkerResults(results, examples, {index, code, signal, done, error}) {
  const completed = new Map(results);
  const missing = examples.filter(name => !completed.get(name)?.length);
  const reason = error?.message || (error ? String(error) :
    `E2E worker ${index + 1} exited code=${code}, signal=${signal ?? 'none'}, done=${done}`);
  const failed = (name) => [{fullName:name,passed:false,skipped:false,duration:0,error:new Error(reason)}];
  for (const name of missing) completed.set(name, failed(`${name} (worker did not report a terminal result)`));
  if (error || code !== 0 || signal || !done) {
    completed.set(`__worker_${index + 1}__`, failed(`worker ${index + 1} lifecycle`));
  }
  return completed;
}
