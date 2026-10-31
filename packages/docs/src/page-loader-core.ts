import type * as Application from 'docs:application';

interface PageResources {
  styles: string[];
  scripts: string[];
}
interface Manifest {
  snapshotId: string;
  pageResources: Record<string, PageResources>;
  resources: { name: string; type: string }[];
  notFound: { outputPath: string }[];
}
export interface PageLoader {
  prepare(key: string): Promise<void>;
  isPublicResource(url: string): Promise<boolean>;
  activate(key?: string): void;
  dispose(): void;
}

/** Browser resource lifecycle; names and ordering come exclusively from Rust. */
export function createPageLoader(
  application: typeof Application,
  initialKey?: string
): PageLoader {
  const { loadPage, site, snapshotId } = application;
  let active = initialKey;
  let manifest: Manifest | undefined;
  let loading: Promise<Manifest> | undefined;
  let observer: MutationObserver | undefined;
  let generation = 0;
  const observedLinks = new WeakMap<HTMLLinkElement, number>();
  const baseStyles = new Set(
    Array.from(
      document.head.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')
    )
      .filter((link) => !link.hasAttribute('data-farm-docs-page-style'))
      .map((link) => link.href)
  );
  const href = (name: string) =>
    new URL(site.basePath + name, location.href).href;

  function reconcile(cleanup = false) {
    if (!manifest) return;
    const known = new Set(
      Object.values(manifest.pageResources).flatMap((page) =>
        page.styles.map(href)
      )
    );
    const desired =
      active === undefined
        ? []
        : (manifest.pageResources[active]?.styles.map(href) ?? []);
    const links = Array.from(
      document.head.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"]')
    );
    const managed = links.filter(
      (link) => known.has(link.href) && !baseStyles.has(link.href)
    );
    for (const link of managed) {
      link.setAttribute('data-farm-docs-page-style', '');
      // Disable the loaded stylesheet, not the link: disabled links can skip
      // fetching, and toggling link.media applies asynchronously in Chromium.
      if (link.sheet) link.sheet.disabled = !desired.includes(link.href);
      else if (observedLinks.get(link) !== generation) {
        const attempt = generation;
        observedLinks.set(link, attempt);
        link.addEventListener(
          'load',
          () => {
            if (attempt === generation) reconcile();
          },
          { once: true }
        );
      }
    }
    const ordered: HTMLLinkElement[] = [];
    for (const url of desired) {
      const matches = managed.filter((link) => link.href === url);
      if (cleanup && matches.length > 1) {
        // Only remove duplicates after prepare resolved: removing an in-flight
        // link would leave the native runtime's onload promise unresolved.
        // The final link is the native loader's completed attempt. An earlier
        // SSR link may have failed before our observers existed, and Chromium
        // can still expose an empty sheet for it. Do not retain that stale copy.
        const completed = matches[matches.length - 1];
        for (const duplicate of matches.slice(0, -1)) duplicate.remove();
        matches.splice(0, matches.length, completed);
      }
      ordered.push(...matches);
    }
    const current = managed.filter(
      (link) => desired.includes(link.href) && link.isConnected
    );
    if (
      current.length !== ordered.length ||
      current.some((link, index) => link !== ordered[index])
    ) {
      for (const link of ordered) document.head.append(link);
    }
  }

  function connect() {
    if (observer) return;
    observer = new MutationObserver(() => reconcile());
    observer.observe(document.head, { childList: true });
    reconcile();
  }

  async function resources() {
    if (!loading) {
      loading = fetch(site.basePath + 'docs-manifest.json')
        .then(async (response) => {
          if (!response.ok)
            throw new Error('[DOCS_MANIFEST] could not load browser resources');
          const value = (await response.json()) as Manifest;
          if (value.snapshotId !== snapshotId || !value.pageResources)
            throw new Error('[DOCS_MANIFEST] browser snapshot mismatch');
          for (const page of Object.values(value.pageResources)) {
            if (
              !Array.isArray(page.styles) ||
              !Array.isArray(page.scripts) ||
              [...page.styles, ...page.scripts].some(
                (name) =>
                  typeof name !== 'string' ||
                  !name ||
                  name.startsWith('/') ||
                  name.includes('\\') ||
                  name.split('/').includes('..')
              )
            ) {
              throw new Error('[DOCS_MANIFEST] invalid page resources');
            }
          }
          manifest = value;
          return value;
        })
        .catch((error) => {
          loading = undefined;
          throw error;
        });
    }
    return loading;
  }

  return {
    async isPublicResource(url) {
      if (generation !== 0)
        throw new Error('[DOCS_NAV_CANCELLED] page loader was disposed');
      const attempt = generation;
      const value = await resources();
      if (attempt !== generation)
        throw new Error('[DOCS_NAV_CANCELLED] page loader was disposed');
      const valid = (name: unknown): name is string =>
        typeof name === 'string' &&
        !!name &&
        !name.startsWith('/') &&
        !/[:\\\x00-\x1f\x7f]/.test(name) &&
        !name.split('/').some((part) => !part || part === '.' || part === '..');
      if (!Array.isArray(value.resources) || !Array.isArray(value.notFound))
        throw new Error('[DOCS_MANIFEST] missing public output registry');
      const names = value.resources.map((resource) => resource?.name);
      names.push(
        ...value.notFound.map((page) => page?.outputPath),
        'docs-manifest.json',
        'docs-build-report.json'
      );
      if (
        names.some((name) => !valid(name)) ||
        new Set(names).size !== names.length
      )
        throw new Error('[DOCS_MANIFEST] invalid public output registry');
      const target = new URL(url, location.href);
      if (target.origin !== location.origin) return false;
      return names.some(
        (name) => new URL(href(name)).pathname === target.pathname
      );
    },
    async prepare(key) {
      const attempt = generation;
      const value = await resources();
      if (attempt !== generation)
        throw new Error('[DOCS_NAV_CANCELLED] page loader was disposed');
      if (!value.pageResources[key])
        throw new Error('[DOCS_MANIFEST] unknown page resources');
      connect();
      await loadPage(key);
      if (attempt !== generation)
        throw new Error('[DOCS_NAV_CANCELLED] page loader was disposed');
    },
    activate(key) {
      active = key;
      reconcile(true);
    },
    dispose() {
      generation++;
      observer?.disconnect();
      observer = undefined;
    }
  };
}
