import type * as Application from 'docs:application';
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import type { PublicDiscoveryLink, PublicPage } from './config.generated.js';
import { NavigationBoundary, RecoveryBoundary } from './navigation-boundary.js';
import { createPageLoader, type PageLoader } from './page-loader-core.js';
import { RuntimeFrame } from './runtime-frame.js';

type Navigation = { key: string | null; url: string; action: 'initial' | 'push' | 'pop'; sequence: number; entry?: string; index?: number; error?: unknown };
type Position = { x: number; y: number };
type Committed = { navigation: Navigation; entry: string; index: number; application: typeof Application; loader: PageLoader };
// URL serialization encodes Unicode exactly as the browser's location does;
// the destination still comes solely from the Rust registry.
function registry(application: typeof Application) {
const { pages, routeAliases, site } = application;
const pagesByKey = new Map(pages.map(page => [page.key, page]));
const routes = new Map<string, PublicPage>();
for (const [path, key] of Object.entries(routeAliases)) {
  const page = pagesByKey.get(key);
  if (!page) throw new Error(`[DOCS_ROUTE_REGISTRY] unknown page: ${key}`);
  const pathname = new URL(path, site.url).pathname;
  const previous = routes.get(pathname);
  if (previous && previous.key !== key) throw new Error(`[DOCS_ROUTE_REGISTRY] ambiguous URL: ${pathname}`);
  routes.set(pathname, page);
}

return { pagesByKey, routes };
}

function updateDiscoveryLinks(links: PublicDiscoveryLink[] = []) {
  document.head.querySelectorAll('link[data-farm-docs-discovery]').forEach(link => link.remove());
  for (const item of links) {
    const link = document.createElement('link');
    link.dataset.farmDocsDiscovery = '';
    link.rel = item.rel;
    link.type = item.type;
    link.title = item.title;
    link.href = item.href;
    document.head.append(link);
  }
}

// Rust owns publication policy and serialization. Transport only the matching
// compiler-generated meta node, including its absence on non-article routes.
function updatePublicationTime(html = '') {
  document.head.querySelectorAll('meta[property="article:published_time"]').forEach(meta => meta.remove());
  const template = document.createElement('template');
  template.innerHTML = html;
  const publication = template.content.querySelector('meta[property="article:published_time"]');
  if (publication) document.head.append(publication.cloneNode(true));
}

export function updateHead(page: PublicPage, application: typeof Application) {
const { seoByKey } = application;
  const head = seoByKey[page.key];
  if (!head || typeof head.title !== 'string') throw new Error(`[DOCS_SEO_HEAD] missing Rust head for ${page.key}`);
  updateDiscoveryLinks(head.discoveryLinks);
  updatePublicationTime(head.html);
  document.title = head.title;
  document.documentElement.lang = page.locale;
  let description = document.head.querySelector<HTMLMetaElement>('meta[name="description"]');
  if (!description) {
    description = document.createElement('meta');
    description.name = 'description';
    document.head.append(description);
  }
  description.content = page.description ?? '';
  let canonical = document.head.querySelector<HTMLLinkElement>('link[rel="canonical"]');
  if (!canonical) {
    canonical = document.createElement('link');
    canonical.rel = 'canonical';
    document.head.append(canonical);
  }
  canonical.href = head.canonicalUrl;
  let robots = document.head.querySelector<HTMLMetaElement>('meta[name="robots"]');
  if (!robots) {
    robots = document.createElement('meta');
    robots.name = 'robots';
    document.head.append(robots);
  }
  robots.content = head.robots;
  // Both targets consume the same Rust policy and already resolved public URLs.
  document.head.querySelectorAll('link[rel="alternate"][hreflang]').forEach(link => link.remove());
  for (const alternate of head.alternates) {
    const link = document.createElement('link');
    link.rel = 'alternate';
    link.hreflang = alternate.locale;
    link.href = alternate.href;
    document.head.append(link);
  }
}

function focusPage(hash: string, position?: Position) {
  // Modal UI must release native inertness before scroll/focus restoration.
  window.dispatchEvent(new Event('farm-docs:before-focus'));
  let id = hash.slice(1);
  try { id = decodeURIComponent(id); } catch { /* Preserve malformed fragments. */ }
  const anchor = id ? document.getElementById(id) : null;
  const target = anchor ?? document.querySelector<HTMLElement>('#farm-docs-content, main, h1');
  if (position) window.scrollTo(position.x, position.y);
  else if (anchor) anchor.scrollIntoView();
  else window.scrollTo(0, 0);
  if (target) {
    if (!target.hasAttribute('tabindex')) target.setAttribute('tabindex', '-1');
    target.focus({ preventScroll: true });
  }
}

export function updateNotFoundHead(application: typeof Application, url: string) {
  const head = application.getNotFound(url);
  updatePublicationTime();
  updateDiscoveryLinks(head.discoveryLinks);
  document.title = head.title;
  document.documentElement.lang = head.locale;
  for (const [name, content] of [['description', head.description], ['robots', head.robots]]) {
    let meta = document.head.querySelector<HTMLMetaElement>(`meta[name="${name}"]`);
    if (!meta) { meta = document.createElement('meta'); meta.name = name; document.head.append(meta); }
    meta.content = content;
  }
  document.head.querySelectorAll('link[rel="canonical"],link[rel="alternate"][hreflang]').forEach(link => link.remove());
}

function RoutePage({ navigation, onCommit, application }: { application: typeof Application; navigation: Navigation; onCommit?: (navigation: Navigation) => void }) {
  const { renderNotFound, renderPage } = application;
  if (navigation.error) throw navigation.error;
  const body = navigation.key === null ? renderNotFound(application.getNotFound(navigation.url).locale) : renderPage(navigation.key);
  useLayoutEffect(() => {
    // Let React finish the commit (including layout-error recovery) before
    // exposing a new URL. Cleanup invalidates abandoned candidates.
    let active = true;
    queueMicrotask(() => { if (active) onCommit?.(navigation); });
    return () => { active = false; };
  }, [navigation, onCommit]);
  return body;
}

/** Browser navigation consumes the Rust registry; it never infers content paths. */
export function BrowserRuntime({ initialKey, loader: providedLoader, application, onGenerationCommit, onGenerationFailure }: { application: typeof Application; initialKey: string | null; loader?: PageLoader; onGenerationCommit?: (key: string | null) => void; onGenerationFailure?: () => void }) {
  const { pagesByKey, routes } = useMemo(() => registry(application), [application]);
  const mounted = (pathname: string) => {
    const base = new URL(application.site.basePath, application.site.url).pathname;
    return pathname === base.slice(0, -1) || pathname.startsWith(base);
  };
  const loader = useMemo(() => providedLoader ?? createPageLoader(application, initialKey ?? undefined), [providedLoader, application]);
  const [navigation, setNavigation] = useState<Navigation>(() => ({
    key: initialKey, url: window.location.href, action: 'initial', sequence: 0
  }));
  const positions = useRef(new Map<string, Position>());
  const [instance] = useState(() => `${Date.now()}-${Math.random()}`);
  const nextEntry = useRef(0);
  const currentEntry = useRef(`${instance}:0`);
  const currentIndex = useRef(0);
  const physicalIndex = useRef(0);
  const latestSequence = useRef(0);
  const committed = useRef<Committed | null>(null);
  const restoring = useRef<Committed | null>(null);
  const queuedNavigation = useRef<Omit<Navigation, 'sequence' | 'error'> | null>(null);
  const [isRestoring, setIsRestoring] = useState(false);
  const [pending, setPending] = useState(false);
  const [loadFailure, setLoadFailure] = useState<Navigation | null>(null);
  const alive = useRef(true);
  const unmountLoader = useRef(loader);
  unmountLoader.current = loader;
  const [generation, setGeneration] = useState(application);
  if (generation !== application) {
    setGeneration(application);
    setNavigation({ key: initialKey, url: location.href, action: 'initial', sequence: ++latestSequence.current });
    setLoadFailure(null);
    setPending(false);
    setIsRestoring(false);
    restoring.current = null;
    queuedNavigation.current = null;
  }
  useEffect(() => () => { alive.current = false; latestSequence.current++; unmountLoader.current.dispose(); }, []);

  const request = (candidate: Omit<Navigation, 'sequence' | 'error'>) => {
    const sequence = ++latestSequence.current;
    setPending(true);
    setLoadFailure(null);
    void (candidate.key === null ? loader.isPublicResource(candidate.url) : loader.prepare(candidate.key).then(() => false)).then(publicResource => {
      if (!alive.current || sequence !== latestSequence.current) return;
      if (publicResource) { location.assign(candidate.url); return; }
      setPending(false);
      setNavigation({ ...candidate, sequence });
    }, error => {
      if (!alive.current || sequence !== latestSequence.current) return;
      setPending(false);
      // Preparing resources has not rendered a candidate. Keep the committed
      // React tree mounted so network failures do not reset its local state.
      setLoadFailure({ ...candidate, sequence, error });
      rollback(sequence);
    });
  };

  const commit = (candidate: Navigation) => {
    if (candidate.sequence !== latestSequence.current || committed.current?.navigation.sequence === candidate.sequence) return;
    const page = candidate.key === null ? undefined : pagesByKey.get(candidate.key);
    if (candidate.key !== null && !page) throw new Error(`[DOCS_ROUTE] unknown page: ${candidate.key}`);
    if (candidate.action === 'push' && candidate.url !== location.href) {
      currentEntry.current = `${instance}:${++nextEntry.current}`;
      currentIndex.current = physicalIndex.current + 1;
      physicalIndex.current = currentIndex.current;
      const previous = history.state && typeof history.state === 'object' ? history.state : {};
      history.pushState({ ...previous, __farmDocsEntry: currentEntry.current, __farmDocsOwner: instance, __farmDocsIndex: currentIndex.current }, '', candidate.url);
    } else if (candidate.action === 'pop') {
      currentEntry.current = candidate.entry!;
      currentIndex.current = candidate.index!;
    } else if (candidate.action === 'push') {
      // A newer click can select the URL of an uncommitted pop. There is no
      // push in that case, but the committed identity must adopt that actual
      // history entry rather than retaining the previously rendered index.
      const state = history.state;
      if (state?.__farmDocsOwner === instance && Number.isInteger(state.__farmDocsIndex) && typeof state.__farmDocsEntry === 'string') {
        currentEntry.current = state.__farmDocsEntry;
        currentIndex.current = state.__farmDocsIndex;
        physicalIndex.current = currentIndex.current;
      }
    }
    committed.current = { navigation: candidate, entry: currentEntry.current, index: currentIndex.current, application, loader };
    loader.activate(candidate.key ?? undefined);
    onGenerationCommit?.(candidate.key);
    if (candidate.action === 'initial') return;
    if (page) updateHead(page, application);
    else updateNotFoundHead(application, candidate.url);
    focusPage(new URL(candidate.url).hash, candidate.action === 'pop' ? positions.current.get(currentEntry.current) : undefined);
  };

  const rollback = (sequence: number) => {
    if (sequence !== latestSequence.current || !committed.current) return;
    const previous = committed.current;
    const delta = previous.index - physicalIndex.current;
    if (delta === 0) return;
    restoring.current = previous;
    setIsRestoring(true);
    history.go(delta);
  };
  const failed = () => { onGenerationFailure?.(); rollback(navigation.sequence); };

  const retry = () => {
    if (restoring.current) return;
    const candidate = loadFailure ?? navigation;
    request({ key: candidate.key, url: candidate.url, action: candidate.action === 'initial' ? 'initial' : 'push' });
  };

  useEffect(() => {
    alive.current = true;
    const previousRestoration = history.scrollRestoration;
    history.scrollRestoration = 'manual';
    const previous = history.state && typeof history.state === 'object' ? history.state : {};
    history.replaceState({ ...previous, __farmDocsEntry: currentEntry.current, __farmDocsOwner: instance, __farmDocsIndex: currentIndex.current }, '', location.href);
    const save = () => positions.current.set(currentEntry.current, { x: scrollX, y: scrollY });
    const navigateUrl = (url: URL) => {
      if (url.origin !== location.origin || url.username || url.password) return false;
      const page = routes.get(url.pathname);
      if (!page && !mounted(url.pathname)) return false;
      if (restoring.current) {
        // history.go is asynchronous. Preserve the latest click until its
        // cursor restoration arrives, then start from that actual position.
        latestSequence.current++;
        queuedNavigation.current = { key: page?.key ?? null, url: url.href, action: 'push' };
        setLoadFailure(null);
        setPending(true);
        return true;
      }
      if (url.href === location.href && committed.current?.navigation.sequence === latestSequence.current) { focusPage(url.hash); return true; }
      save();
      request({ key: page?.key ?? null, url: url.href, action: 'push' });
      return true;
    };
    const click = (event: MouseEvent) => {
      if (event.defaultPrevented || event.button !== 0 || event.metaKey || event.ctrlKey || event.shiftKey || event.altKey) return;
      const anchor = event.target instanceof Element ? event.target.closest<HTMLAnchorElement>('a[href]') : null;
      if (!anchor || anchor.hasAttribute('download') || anchor.relList.contains('external') || (anchor.target && anchor.target !== '_self')) return;
      if (navigateUrl(new URL(anchor.href, location.href))) event.preventDefault();
    };
    const programmatic = (event: Event) => {
      if (event.defaultPrevented || !(event instanceof CustomEvent) || typeof event.detail?.url !== 'string') return;
      const url = new URL(event.detail.url, location.href);
      if (navigateUrl(url)) event.preventDefault();
    };
    const pop = (event: PopStateEvent) => {
      const page = routes.get(location.pathname);
      if (!mounted(location.pathname) || event.state?.__farmDocsOwner !== instance || !Number.isInteger(event.state?.__farmDocsIndex) || typeof event.state?.__farmDocsEntry !== 'string') { location.reload(); return; }
      physicalIndex.current = event.state.__farmDocsIndex;
      if (restoring.current?.entry === event.state?.__farmDocsEntry) {
        restoring.current = null;
        setIsRestoring(false);
        const queued = queuedNavigation.current;
        queuedNavigation.current = null;
        if (queued) request(queued);
        else focusPage(new URL(location.href).hash, positions.current.get(currentEntry.current));
        return;
      }
      restoring.current = null;
      queuedNavigation.current = null;
      setIsRestoring(false);
      request({ key: page?.key ?? null, url: location.href, action: 'pop', entry: event.state.__farmDocsEntry, index: event.state.__farmDocsIndex });
    };
    document.addEventListener('click', click);
    window.addEventListener('farm-docs:navigate', programmatic);
    window.addEventListener('popstate', pop);
    window.addEventListener('scroll', save, { passive: true });
    return () => {
      alive.current = false;
      queuedNavigation.current = null;
      if (!providedLoader) loader.dispose();
      document.removeEventListener('click', click);
      window.removeEventListener('farm-docs:navigate', programmatic);
      window.removeEventListener('popstate', pop);
      window.removeEventListener('scroll', save);
      history.scrollRestoration = previousRestoration;
    };
  }, [application, loader]);
  return <RuntimeFrame content={<NavigationBoundary sequence={navigation.sequence} onFailure={failed} onRetry={retry} restoring={isRestoring}
    fallback={() => {
      // Read at failure time: child state may fail after this route committed,
      // without another BrowserRuntime render to refresh captured props.
      const previous = committed.current;
      return previous ? <RecoveryBoundary key={previous.navigation.sequence}><RoutePage application={previous.application} navigation={previous.navigation} /></RecoveryBoundary> : null;
    }}>
    <RoutePage application={application} navigation={navigation} onCommit={commit} />
  </NavigationBoundary>} failure={loadFailure ? <aside role="alert">
    <p>Could not load this page. You can retry the navigation.</p>
    <button type="button" disabled={isRestoring} onClick={retry}>Retry navigation</button>
  </aside> : null} pending={pending ? <p role="status">Loading page…</p> : null} />;
}
