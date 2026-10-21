import { getNotFound, snapshotId } from 'docs:application';
import { hydrateRoot } from 'react-dom/client';
import { BrowserRuntime } from './browser-runtime.js';
import { createPageLoader } from './page-loader.js';

const data = document.getElementById('__FARM_DOCS_DATA__');
const container = document.getElementById('__FARM_DOCS_ROOT__');
if (!data || !container) throw new Error('[DOCS_HYDRATION] missing page state or root');
const hydrationRoot = container;
const state: { snapshotId: string; key: string; kind?: never } | { snapshotId: string; kind: 'notFound'; locale: string } = JSON.parse(data.textContent ?? '');
if (state.snapshotId !== snapshotId) throw new Error('[DOCS_HYDRATION] page and client snapshot differ');
if (state.kind === 'notFound' && (typeof state.locale !== 'string' || state.locale !== getNotFound(location.href).locale)) throw new Error('[DOCS_HYDRATION] NotFound locale differs from Rust route selection');
// Keep readable SSR content until its actual browser module is ready.
let failure: HTMLElement | undefined;
const initialKey = state.kind === 'notFound' ? null : state.key;
const loader = createPageLoader(initialKey ?? undefined);
async function start() {
  try {
    if (initialKey !== null) await loader.prepare(initialKey);
    failure?.remove();
    hydrateRoot(hydrationRoot, <BrowserRuntime initialKey={initialKey} loader={loader} />);
  } catch {
    if (!failure) {
      failure = document.createElement('aside');
      failure.setAttribute('role', 'alert');
      failure.append('Could not load this page. ');
      const retry = document.createElement('button');
      retry.type = 'button';
      retry.textContent = 'Retry navigation';
      retry.onclick = () => { retry.disabled = true; void start().finally(() => { retry.disabled = false; }); };
      failure.append(retry);
      container?.after(failure);
    }
  }
}
void start();
