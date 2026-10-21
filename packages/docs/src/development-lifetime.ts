import { inspect } from 'node:util';

// Process lifecycle only. Compilation remains owned by native plugins.
const owners = new Set<() => Promise<void>>();
let terminating = false;
let installed = false;
let watchStdin = false;

function uninstall(): void {
  if (!installed) return;
  installed = false;
  process.off('SIGINT', terminate);
  process.off('SIGTERM', terminate);
  if (watchStdin) process.stdin.off('end', terminate);
}

function terminate(): void {
  if (terminating) return;
  terminating = true;
  // A close failure must not prevent other registered owners from draining.
  void Promise.allSettled(
    [...owners].map((close) => Promise.resolve().then(close))
  ).then((results) => {
    for (const result of results) {
      if (result.status === 'rejected') {
        console.error(
          '[DOCS_DEV_CLOSE]',
          inspect(result.reason, { depth: null })
        );
        process.exitCode = 1;
      }
    }
    uninstall();
    process.exit();
  });
}

/** Keep one signal handler until every registered Docs owner has drained. */
export function registerDevelopmentLifetime(
  close: () => Promise<void>
): () => void {
  if (terminating)
    throw new Error('[DOCS_DEV_CLOSED] process termination is in progress');
  owners.add(close);
  if (!installed) {
    installed = true;
    watchStdin = process.env.CI !== 'true';
    process.on('SIGINT', terminate);
    process.on('SIGTERM', terminate);
    if (watchStdin) process.stdin.on('end', terminate);
  }
  return () => {
    owners.delete(close);
    if (!owners.size && !terminating) uninstall();
  };
}
