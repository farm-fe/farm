import * as initialApplication from 'docs:application';
import { beginRefreshTransaction, flushRefresh } from 'farm:react-refresh';
import { createRoot } from 'react-dom/client';
import { BrowserRuntime, updateHead, updateNotFoundHead } from './browser-runtime-core.js';
import { createPageLoader, type PageLoader } from './page-loader-core.js';
import { restoreTogether, validateTogether, type DevelopmentTransaction } from './development-transaction.js';

const container = document.getElementById('__FARM_DOCS_ROOT__');
if (!container) throw new Error('[DOCS_DEVELOPMENT] missing root');
const root = createRoot(container);
let currentLoader: PageLoader | undefined;
let requestedGeneration = 0;
let committedApplication = initialApplication;
let recoveryEpoch = 0;
type Candidate = { publish(): void; dispose(): void };
type Batch = { candidate?: Candidate; failure?: Error };
let batch: Batch | undefined;

async function mount(application = committedApplication, retainedLoader?: PageLoader) {
  const generation = ++requestedGeneration;
  const { pages, routeAliases, site } = application;
  const routeKey = () => Object.entries(routeAliases).find(([route]) => new URL(route, site.url).pathname === location.pathname)?.[1] ?? null;
  let key = routeKey();
  const loader = retainedLoader ?? createPageLoader(application, key ?? undefined);
  try {
    let preparedUrl: string;
    do {
      preparedUrl = location.href;
      key = routeKey();
      if (key !== null) await loader.prepare(key);
      // Preparing a lazy wrapper can evaluate new content after the accept callback's earlier flush.
      await flushRefresh();
      // Navigation may commit while this candidate waits for its Rust manifest/resources.
      if (preparedUrl === location.href) break;
    } while (generation === requestedGeneration);
  }
  catch (error) { if (!retainedLoader) loader.dispose(); throw error; }
  if (generation !== requestedGeneration) { if (!retainedLoader) loader.dispose(); return; }
  let committedKey = key;
  await new Promise<void>((resolve, reject) => {
    let settled = false;
    root.render(<BrowserRuntime key={recoveryEpoch} initialKey={key} loader={loader} application={application}
      onGenerationCommit={pageKey => {
        committedKey = pageKey;
        if (settled) return;
        settled = true;
        if (generation !== requestedGeneration) { loader.dispose(); resolve(); return; }
        const candidate: Candidate = {
          publish() {
            const previous = currentLoader;
            currentLoader = loader;
            committedApplication = application;
            const page = committedKey === null ? undefined : pages.find(page => page.key === committedKey);
            if (page) updateHead(page, application);
            else updateNotFoundHead(application, location.href);
            if (previous !== loader) previous?.dispose();
            document.getElementById('farm-docs-development-error')?.remove();
          },
          dispose() { if (!retainedLoader) loader.dispose(); }
        };
        if (batch) batch.candidate = candidate;
        else candidate.publish();
        resolve();
      }}
      onGenerationFailure={() => {
        // A self-accepted nested component can fail the already committed page without mounting a new application.
        if (batch) batch.failure ??= new Error('[DOCS_DEVELOPMENT_RENDER] active page failed during update');
        if (settled) return;
        settled = true;
        if (!retainedLoader) loader.dispose();
        reject(new Error('[DOCS_DEVELOPMENT_RENDER] candidate could not render'));
      }} />);
  });
}

declare global {
  interface ImportMeta {
    readonly hot?: {
      on(event: string, callback: (payload: { snapshotId?: unknown }) => void | Promise<void>): void;
      send(event: string, payload?: unknown): void;
      accept(dependency: string, callback: (application: typeof initialApplication) => Promise<void>): void;
      registerUpdateParticipant(prepare: (modules: { beginResourceTransaction(): Promise<DevelopmentTransaction> }) => Promise<{
        validate(): Promise<void>; commit(): void; rollback(): Promise<void>; recover(): Promise<void>;
      }>): () => void;
    };
  }
}
if (import.meta.hot) {
  const hot = import.meta.hot;
  let reloadingGeneration = false;
  const synchronize = () => hot.send('farm-docs:sync');
  hot.on('farm-docs:sync', async ({ snapshotId }) => {
    if (typeof snapshotId !== 'string' || !snapshotId || reloadingGeneration) return;
    await initialReady;
    const storageKey = `farm-docs-generation:${location.pathname}`;
    if (snapshotId === committedApplication.snapshotId) {
      if (currentLoader) {
        try { sessionStorage.removeItem(storageKey); } catch { /* Storage may be disabled. */ }
      }
      return;
    }
    const retry = () => {
      try { sessionStorage.removeItem(storageKey); } catch { /* Manual refresh remains available. */ }
      location.reload();
    };
    try {
      if (sessionStorage.getItem(storageKey) === snapshotId) {
        showInitialError(new Error('[DOCS_DEVELOPMENT_SYNC] Could not load the current generation.'), retry);
        return;
      }
      sessionStorage.setItem(storageKey, snapshotId);
    } catch {
      showInitialError(new Error('[DOCS_DEVELOPMENT_SYNC] Refresh to load the current generation.'), retry);
      return;
    }
    reloadingGeneration = true;
    location.reload();
  });
  hot.on('farm:ws:connect', synchronize);
  // send is ignored before OPEN; the connect listener covers that case. If
  // OPEN happened before module evaluation, this request covers the lost event.
  synchronize();
  import.meta.hot.registerUpdateParticipant(async modules => {
    if (!currentLoader) await initialReady;
    await flushRefresh();
    const resources = await modules.beginResourceTransaction();
    let refresh: Awaited<ReturnType<typeof beginRefreshTransaction>>;
    try { refresh = await beginRefreshTransaction(); }
    catch (error) { await resources.settle(); resources.restore(); throw error; }
    const previousApplication = committedApplication, previousLoader = currentLoader;
    const candidateBatch: Batch = {};
    batch = candidateBatch;
    const transactions = [refresh, resources];
    return {
      async validate() {
        await validateTogether(transactions);
        if (candidateBatch.failure) throw candidateBatch.failure;
      },
      ready() { return !candidateBatch.failure && transactions.every(transaction => transaction.isSettled()); },
      commit() {
        for (const transaction of transactions) transaction.finalize();
        batch = undefined;
        candidateBatch.candidate?.publish();
      },
      async rollback() {
        requestedGeneration++;
        try { await restoreTogether(transactions); }
        finally { candidateBatch.candidate?.dispose(); batch = undefined; }
      },
      async recover() {
        recoveryEpoch++;
        await mount(previousApplication, previousLoader);
        const alert = document.createElement('aside');
        alert.id = 'farm-docs-development-error';
        alert.setAttribute('role', 'alert');
        alert.textContent = 'Update failed. Showing the last working page.';
        container!.after(alert);
      }
    };
  });
  import.meta.hot.accept('docs:application', async application => {
    await flushRefresh(); await mount(application);
  });
}

function showInitialError(error: unknown, retryLoading?: () => void) {
  document.getElementById('farm-docs-development-error')?.remove();
  const alert = document.createElement('aside');
  alert.id = 'farm-docs-development-error';
  alert.setAttribute('role', 'alert');
  alert.textContent = error instanceof Error ? error.message : String(error);
  const retry = document.createElement('button');
  retry.textContent = 'Retry loading';
  retry.onclick = () => {
    if (retryLoading) { retryLoading(); return; }
    alert.remove();
    void mount().catch((failure: unknown) => {
      alert.firstChild!.textContent = failure instanceof Error ? failure.message : String(failure);
      container!.after(alert);
    });
  };
  alert.append(retry);
  container!.after(alert);
}
const initialReady = mount().catch((error: unknown) => showInitialError(error));
