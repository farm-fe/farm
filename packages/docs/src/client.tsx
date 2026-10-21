import type { AnchorHTMLAttributes, ReactNode } from 'react';
import { createContext, useContext, useEffect, useState } from 'react';
import { type ColorModeState, useColorModeState } from './color-mode.js';
import type { PublicPage, PublicSiteConfig } from './config.generated.js';

export type { ColorModeState } from './color-mode.js';
export type { ColorMode } from './config.generated.js';

interface DocsContextValue { site: PublicSiteConfig; page: PublicPage | null; locale: string; colorMode: ColorModeState }
const DocsContext = createContext<DocsContextValue | null>(null);

export function DocsProvider({ site, page, children }: { site: PublicSiteConfig; page: PublicPage; children: ReactNode }) {
  const colorMode = useColorModeState(site.theme.colorMode, site.basePath);
  return <DocsContext.Provider value={{ site, page, locale: page.locale, colorMode }}>{children}</DocsContext.Provider>;
}
/** Missing-page rendering has site state but deliberately has no content page. */
export function NotFoundProvider({ site, locale, children }: { site: PublicSiteConfig; locale: string; children: ReactNode }) {
  const colorMode = useColorModeState(site.theme.colorMode, site.basePath);
  return <DocsContext.Provider value={{ site, page: null, locale, colorMode }}>{children}</DocsContext.Provider>;
}
function useDocs() {
  const value = useContext(DocsContext);
  if (!value) throw new Error('[DOCS_CONTEXT] component requires DocsProvider');
  return value;
}
export function useSite() { return useDocs().site; }
export function usePage() {
  const page = useDocs().page;
  if (!page) throw new Error('[DOCS_PAGE_CONTEXT] missing-page views have no content page');
  return page;
}
export function useLocale() { return useDocs().locale; }
export function useVersion() { return usePage().version; }
export function useColorMode() { return useDocs().colorMode; }
export function Link(props: AnchorHTMLAttributes<HTMLAnchorElement> & { to?: string }) {
  const { to, href, ...rest } = props;
  return <a {...rest} href={to ?? href} />;
}
/** Navigate through the mounted Docs runtime, falling back to a document load. */
export function navigate(to: string) {
  const url = new URL(to, window.location.href);
  if (!['http:', 'https:'].includes(url.protocol)) throw new Error('[DOCS_NAVIGATION_URL] expected an HTTP URL');
  const event = new CustomEvent('farm-docs:navigate', { cancelable: true, detail: { url: url.href } });
  if (window.dispatchEvent(event)) window.location.assign(url.href);
}
export function ClientOnly({ children, fallback = null }: { children: () => ReactNode; fallback?: ReactNode }) {
  const [mounted, setMounted] = useState(false);
  useEffect(() => setMounted(true), []);
  return <>{mounted ? children() : fallback}</>;
}
